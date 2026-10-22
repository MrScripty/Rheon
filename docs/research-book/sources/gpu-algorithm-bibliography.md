# GPU grid algorithm bibliography and source audit

This is the source supplement for [Chapter 37](../chapters/37-gpu-grid-algorithms.md) of the Puma-credited Rheon research book. Access date for every successful retrieval below: **2026-10-21**. “Verified” means the identified content or source file was inspected through read-only web retrieval; it does not mean its implementation was built, executed, benchmarked, or independently numerically validated. Local design deductions in Chapter 37 are identified as such and do not acquire validation from a citation.

The scope of each entry matters. A paper can motivate an algorithm without proving its suitability for Rheon's boundary-value problem. A published timing remains a result for the authors' implementation, machine and stopping criterion. A source repository supplies inspectable implementation evidence, with separately identified version and license; no source was incorporated into Rheon in this research change.

## G1 — Harris: grid passes on an early GPU

**Mark J. Harris.** “Fast Fluid Dynamics Simulation on the GPU.” *GPU Gems*, chapter 38, 2004. [Publisher-hosted chapter](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu).

Inspected the full HTML chapter, especially sections 38.1.3 and 38.3. It describes a two-dimensional, homogeneous rectangular-domain stable-fluids implementation using GPU texture passes. It explicitly excludes free surfaces from that treatment. Used for the historical decomposition into independent grid operations; its old GPU timings and texture/API constraints are not current Rheon evidence. No accompanying sample software was downloaded or licensed for incorporation.

## G2 — Crane, Llamas and Tariq: three-dimensional visual-effects pipeline

**Keenan Crane, Ignacio Llamas and Sarah Tariq.** “Real-Time Simulation and Rendering of 3D Fluids.” *GPU Gems 3*, chapter 30, 2007. [Publisher-hosted chapter](https://developer.nvidia.com/gpugems/gpugems3/part-v-physics-simulation/chapter-30-real-time-simulation-and-rendering-3d-fluids).

Inspected HTML sections 30.2.3 onward for grid storage, simulation passes and corrected advection. The chapter explicitly uses cell-centered quantities for exposition while identifying staggered MAC discretization as more robust. Used for GPU pipeline history and as a warning against importing a collocated sample's pressure stencil into Rheon's MAC operator. Its fluid/rendering examples do not establish conservation or boundary correctness for the proposed backend.

## G3 — Bolz et al.: sparse CG and regular-grid multigrid

**Jeff Bolz, Ian Farmer, Eitan Grinspun and Peter Schröder.** “Sparse Matrix Solvers on the GPU: Conjugate Gradients and Multigrid.” *ACM Transactions on Graphics* 22(3), 917–924, 2003. DOI: [10.1145/882262.882364](https://doi.org/10.1145/882262.882364). [Caltech-hosted paper PDF](https://multires.caltech.edu/pubs/GPUSim.pdf).

Verified the eight-page primary PDF, including its solver formulation, stream/gather organization and reductions. Its GPU work includes sparse CG and regular-grid multigrid. Used to distinguish local vector/stencil work from global scalar dependencies. The text's GeForce FX execution model is historical. Direct ACM retrieval returned HTTP 403; the Caltech primary paper supplied the inspected content. The initially tried `gpusolver.pdf` address was unavailable and is not a supporting source.

## G4 — McAdams, Sifakis and Teran: parallel multigrid preconditioning

**Aleka McAdams, Eftychios Sifakis and Joseph Teran.** “A Parallel Multigrid Poisson Solver for Fluids Simulation on Large Grids.” *ACM SIGGRAPH/Eurographics Symposium on Computer Animation*, 2010. [Author/institution publication record](https://graphics.cs.wisc.edu/Papers/2010/MST10/). [Author-hosted full paper PDF](https://www.math.ucdavis.edu/~jteran/papers/MST10.pdf).

Verified the ten-page PDF and publication metadata. The work combines a lightweight geometric multigrid cycle with CG for irregular voxelized pressure domains and mixed Neumann/Dirichlet conditions. Its implementation and reported measurements target **shared-memory CPU platforms**. Used for pressure algorithm design, hierarchy overhead and boundary-aware preconditioning; it is not labeled a GPU implementation or a GPU benchmark in this chapter.

## G5 — Harris: hierarchical reductions

**Mark Harris.** *Optimizing Parallel Reduction in CUDA*. NVIDIA Developer Technology presentation. [NVIDIA-hosted 38-page PDF](https://developer.download.nvidia.com/assets/cuda/files/reduction.pdf).

Verified the PDF title, author, tree reduction, partial-result decomposition and optimization discussion. A publication date is not asserted because the inspected title slide does not establish one. Used only for the historical reduction construction and bandwidth motivation. Old hardware-specific claims about global synchronization and warp execution are not used as current CUDA or Vulkan normative requirements. Backend synchronization is governed by the separately reviewed API sources in Chapter 38.

## G6 — Bridson and Müller-Fischer: pressure, MIC and boundary derivations

**Robert Bridson and Matthias Müller-Fischer.** *Fluid Simulation: SIGGRAPH 2007 Course Notes*. Title page dated **10 August 2007**. [University of British Columbia full PDF](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf).

Verified the 93-page PDF, its title page and contents including sections 4.3.2–4.3.4 on CG, incomplete Cholesky and modified incomplete Cholesky, plus curved pressure boundaries. Used as the primary course-note reference for CPU-oriented preconditioning and discrete fluid semantics. A first retrieval timed out; a later full PDF retrieval succeeded. The notes are not a GPU parallel triangular-solve implementation.

## G7 — Weber et al.: cut-cell multigrid as geometry-specific research

**Daniel Weber, Johannes Mueller-Roemer, André Stork and Dieter Fellner.** “A Cut-Cell Geometric Multigrid Poisson Solver for Fluid Simulation.” *Computer Graphics Forum* **34(2)**, 481–491, 2015. DOI: [10.1111/cgf.12577](https://doi.org/10.1111/cgf.12577). [Publisher article](https://onlinelibrary.wiley.com/doi/full/10.1111/cgf.12577). [Author institute publication listing](https://www.igd.fraunhofer.de/en/institute/employees/daniel-weber.html).

Verified publisher-indexed metadata and author-institute listing; also inspected the primary author-manuscript text in a [hosted manuscript copy](https://www.academia.edu/14777152/A_Cut_Cell_Geometric_Multigrid_Poisson_Solver_for_Fluid_Simulation). The limited use here is its staggered finite-volume cut-cell hierarchy motivation. No numerical rate or GPU timing is imported. Direct publisher opening timed out/returned 403; the Fraunhofer repository record was protected by an access-denial page. Those failed fetches are not represented as full-text verification. Some institutional indexing displays volume 35; the publisher citation and manuscript identify volume **34**, used here.

## G8 — NVIDIA AmgX: inspected GPU linear-solver source

**NVIDIA. AmgX.** Repository [NVIDIA/AMGX](https://github.com/NVIDIA/AMGX). Inspected commit **`91a8413ef267b1c32aff4014c02820e1c5897ac2`**, GitHub commit timestamp **2026-07-09T07:00:23Z**. The pinned [ReleaseVersion.txt](https://github.com/NVIDIA/AMGX/blob/91a8413ef267b1c32aff4014c02820e1c5897ac2/ReleaseVersion.txt) reads **2.5.0**. This identifies the inspected snapshot; it is not a claim that the commit is a version-release date.

Inspected through raw source retrieval:

- [PCG implementation](https://github.com/NVIDIA/AMGX/blob/91a8413ef267b1c32aff4014c02820e1c5897ac2/src/solvers/pcg_solver.cu): operator application, inner products, vector updates and preconditioner invocation.
- [Polynomial smoother source](https://github.com/NVIDIA/AMGX/blob/91a8413ef267b1c32aff4014c02820e1c5897ac2/src/solvers/chebyshev_poly.cu): an implementation reference, not an endorsed parameter selection.
- [Example PCG/aggregation/Jacobi configuration](https://github.com/NVIDIA/AMGX/blob/91a8413ef267b1c32aff4014c02820e1c5897ac2/src/configs/PCG_AGGREGATION_JACOBI.json): demonstrates composition; its symmetry or suitability is not inferred from the outer solver name.
- [BSD 3-Clause license text](https://github.com/NVIDIA/AMGX/blob/91a8413ef267b1c32aff4014c02820e1c5897ac2/LICENSES/BSD-3-Clause.txt), plus source SPDX header on the polynomial smoother.

License conclusion is limited to the inspected project license/source headers; bundled dependencies have their own notices. This is a CUDA solver-library comparison, not a Vulkan solution, full fluid pipeline, or proposed mandatory Rheon dependency. No library build, installation or execution occurred.

## G8b — NVIDIA Flow in PhysX: inspectable sparse pressure passes

**NVIDIA. Flow SDK**, in [NVIDIA-Omniverse/PhysX](https://github.com/NVIDIA-Omniverse/PhysX). Inspected commit **`b97365c6ddb32fda8469bc788ec18024862b0368`**, GitHub commit timestamp **2026-10-15T11:12:53Z**. The commit is the reproducible version identifier; no independent Flow semantic-version number was established.

Verified these pinned files via raw source retrieval:

- [Pressure pass orchestration](https://github.com/NVIDIA-Omniverse/PhysX/blob/b97365c6ddb32fda8469bc788ec18024862b0368/flow/source/nvflow/Pressure.cpp): separate divergence, Jacobi, residual, restriction, prolongation and subtraction pipelines, plus pressure-level resources.
- [Jacobi shader](https://github.com/NVIDIA-Omniverse/PhysX/blob/b97365c6ddb32fda8469bc788ec18024862b0368/flow/source/nvflow/shaders/PressureJacobiCS.hlsl): six-neighbor pressure texture reads, table-based addressing and distinct output.
- [Divergence shader](https://github.com/NVIDIA-Omniverse/PhysX/blob/b97365c6ddb32fda8469bc788ec18024862b0368/flow/source/nvflow/shaders/PressureDivergenceCS.hlsl): centered differences of collocated `Texture3D<float4>` velocity components. This explicitly limits comparability with Rheon's MAC discretization.
- [Root BSD 3-Clause license](https://github.com/NVIDIA-Omniverse/PhysX/blob/b97365c6ddb32fda8469bc788ec18024862b0368/LICENSE.md), BSD-3-Clause SPDX headers on inspected pressure files, and [Flow README with third-party acknowledgments](https://github.com/NVIDIA-Omniverse/PhysX/blob/b97365c6ddb32fda8469bc788ec18024862b0368/flow/README.md).

This modern source snapshot is distinct from the older `NVIDIAGameWorks/Flow` repository and its license. A guessed `LICENSE.md` in that older repository returned 404 and is not evidence for its license. No claim is made that the inspected shaders implement the integrated MAC pressure equation or physically general liquid coupling. No Flow binary was built or run.

## G9 — Aanjaneya et al.: coupled viscous multigrid

**Mridul Aanjaneya, Chengguizi Han, Ryan Goldade and Christopher Batty.** “An Efficient Geometric Multigrid Solver for Viscous Liquids.” *Proceedings of the ACM on Computer Graphics and Interactive Techniques* 2(2), article 14, 2019. DOI: [10.1145/3340255](https://doi.org/10.1145/3340255). [Primary author project page and abstract](https://orionquest.github.io/papers/EGMVL/paper.html).

Verified the author page's authorship, venue, year and abstract describing a box smoother for coupled viscous stencils and multigrid preconditioning. The full [author-hosted PDF](https://cs.uwaterloo.ca/~c2batty/papers/Aanjaneya2019/Aanjaneya2019.pdf) exceeded the web tool's content-size limit; direct ACM opening returned 403. Therefore this entry supports the narrow abstract-level point about coupled smoothers, not a detailed reproduction of the paper or an implementation audit. No published performance figure is transferred to Rheon.

## Retrieval and qualification limits

All repository hashes and dates above came from read-only GitHub API metadata; source contents and licenses were fetched at those hashes. API default branches were used only to resolve a snapshot and are not the cited version identity. Project license texts are distinguished from licenses of bundled third-party material. A final implementation decision must review the exact source selected for reuse and retain its notices.

The inspected sources span graphics-oriented stable-fluids examples, pressure linear algebra, parallel CPU multigrid and GPU solver source. Particle-only solvers, lattice-Boltzmann methods and appearance voxel grids are not treated as evidence of MAC projection compatibility. This bibliography intentionally makes no unsupported claim that a particular package offers identical independently maintained CUDA and Vulkan fluid solvers.

The research session performed source and documentation inspection only. It did not install drivers/toolchains, use GPU hardware, execute scientific kernels, run fluid fixtures, reproduce paper benchmarks, or run the full research-book build. Remaining scientific and performance qualification belongs to [Chapter 39](../chapters/39-gpu-validation-and-roadmap.md).
