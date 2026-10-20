# C Primary sources

The works below provide historical and technical context. Graph definitions, worked arithmetic, recommendations, experiments and Lean contracts are independently specified for Rheon. No third-party figures or long quotations are reproduced. Publisher-only citations identify original works without implying full-paper reproduction.

[S1] Jos Stam. Stable Fluids. SIGGRAPH 1999, 121-128. DOI 10.1145/311535.311548. [Author paper](https://www.dgp.toronto.edu/public_user/stam/reality/Research/pdf/ns.pdf). Semi-Lagrangian graphics simulation.

[S2] Francis H. Harlow and J. Eddie Welch. Numerical Calculation of Time-Dependent Viscous Incompressible Flow of Fluid with Free Surface. Physics of Fluids 8, 2182-2189, 1965. DOI 10.1063/1.1761178. [Paper](https://www.cs.rpi.edu/~cutler/classes/advancedgraphics/S10/papers/harlow_welch.pdf). Marker-and-cell foundation.

[S3] Alexandre J. Chorin. Numerical Solution of the Navier-Stokes Equations. Mathematics of Computation 22, 745-762, 1968. [Author paper](https://math.berkeley.edu/~chorin/chorin68.pdf). Projection methods.

[S4] Robert Bridson and Matthias Müller-Fischer. Fluid Simulation. SIGGRAPH 2007 Course Notes. [Author notes](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf). Broad numerical implementation context.

[S5] Jonathan Richard Shewchuk. An Introduction to the Conjugate Gradient Method Without the Agonizing Pain. 1994. [Author paper](https://www.cs.cmu.edu/~quake-papers/painless-conjugate-gradient.pdf). Krylov methods and conditioning.

[S6] Andrew Selle, Ronald Fedkiw, ByungMoon Kim, Yingjie Liu and Jarek Rossignac. An Unconditionally Stable MacCormack Method. Journal of Scientific Computing 35, 350-371, 2008. [Publisher](https://doi.org/10.1007/s10915-007-9166-4). Corrected advection.

[S7] Christopher Batty, Florence Bertails and Robert Bridson. A Fast Variational Framework for Accurate Solid-Fluid Coupling. ACM Transactions on Graphics 26(3), article 100, 2007. [Author paper](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf). Sections 2.1 and 3: alternative staggered masses, adjoint pressure/body coupling and the thick-body limitation. Author PDF re-read for the 20 October 2026 audit; the former lab-path PDF was unavailable.

[S8] Stanley Osher and James A. Sethian. Fronts Propagating with Curvature-Dependent Speed: Algorithms Based on Hamilton-Jacobi Formulations. Journal of Computational Physics 79, 12-49, 1988. [Author paper](https://math.berkeley.edu/~sethian/2006/Papers/sethian.osher.88.pdf). Level sets.

[S9] C. W. Hirt and B. D. Nichols. Volume of Fluid Method for the Dynamics of Free Boundaries. Journal of Computational Physics 39, 201-225, 1981. [Publisher](https://www.sciencedirect.com/science/article/pii/0021999181901455). Volume fractions.

[S10] Douglas Enright, Ronald Fedkiw, Joel Ferziger and Ian Mitchell. A Hybrid Particle Level Set Method for Improved Interface Capturing. Journal of Computational Physics 183, 83-116, 2002. [Publisher](https://www.sciencedirect.com/science/article/pii/S0021999102971664). Particle interface correction.

[S11] Christopher Batty and Robert Bridson. A Simple Finite Difference Method for Time-Dependent, Variable Coefficient Stokes Flow on Irregular Domains. Preprint 2010, revised 12 May 2011, arXiv:1010.2832v2. [Primary text](https://arxiv.org/html/1010.2832v2), [version record](https://arxiv.org/abs/1010.2832v2). Sections 2–3 and 6 distinguish the full traction boundary from a split pressure/viscosity approximation. Re-read for the 20 October 2026 audit; the article's results are not inherited Rheon results.

[S12] Ronald Fedkiw, Jos Stam and Henrik Wann Jensen. Visual Simulation of Smoke. SIGGRAPH 2001. [Author paper](https://physbam.stanford.edu/papers/stanford2001-01.pdf). Smoke and confinement.

[S13] Matthias Müller, David Charypar and Markus Gross. Particle-Based Fluid Simulation for Interactive Applications. SCA 2003. [Author paper](https://matthias-research.github.io/pages/publications/sca03.pdf). SPH.

[S14] Lvmin Zhang, Anyi Rao and Maneesh Agrawala. Adding Conditional Control to Text-to-Image Diffusion Models. 2023. [Paper](https://arxiv.org/abs/2302.05543). Spatial conditioning; not a Rheon-specific error guarantee.

[S15] Miles Macklin and Matthias Müller. Position Based Fluids. ACM Transactions on Graphics 32, 2013. [Author preprint](https://matthias-research.github.io/pages/publications/pbf_sig_preprint.pdf). Density constraints.

[S16] Chenfanfu Jiang, Craig Schroeder, Andrew Selle, Joseph Teran and Alexey Stomakhin. The Affine Particle-In-Cell Method. ACM Transactions on Graphics 34, 2015. [Publisher](https://doi.org/10.1145/2766996). APIC.

[S17] Chenfanfu Jiang, Craig Schroeder and Joseph Teran. An Angular Momentum Conserving Affine-Particle-In-Cell Method. 2016 preprint; Journal of Computational Physics, 2017. [Paper](https://arxiv.org/abs/1603.06188). Specified transfer conservation.

[S18] David Goldberg. What Every Computer Scientist Should Know About Floating-Point Arithmetic. ACM Computing Surveys, 1991. [Authorized reprint](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html). Rounding and exceptional arithmetic.

[S19] Rheon repository README, revision 0cb48429789699578b640d30d9d5dc99c6921ce4. [Pinned source](https://github.com/MrScripty/Rheon/blob/0cb48429789699578b640d30d9d5dc99c6921ce4/README.md). Project scope; no production solver at this revision.

[S20] MrScripty. Coding Standards, revision dcc56f26e884ade260770beceba2501d3746200d. [Pinned source](https://github.com/MrScripty/Coding-Standards/tree/dcc56f26e884ade260770beceba2501d3746200d). Ownership, evidence and performance guidance.

[S21] Lean community. [Lean 4.19.0](https://github.com/leanprover/lean4/releases/tag/v4.19.0) and [pinned mathlib](https://github.com/leanprover-community/mathlib4/tree/c44e0c8ee63ca166450922a373c7409c5d26b00b). Formal environment.

[S22] Yousef Saad. Iterative Methods for Sparse Linear Systems, second edition. SIAM, 2003. [Author hosted book](https://www-users.cse.umn.edu/~saad/IterMethBook_2ndEd.pdf). Multigrid and iterative-method context. The companion hierarchy and its measured results are independently constructed.

## Additional primary sources for the liquid–solid audit

Consulted 20 October 2026. These entries state the portion actually inspected;
an author abstract is used only for high-level method comparison. No secondary
summary supplies mathematical authority, and no third-party figure is reproduced.

[A1] CGAL developers. Polygon Mesh Processing, official manual, release 6.2.1.
[Manual](https://doc.cgal.org/latest/Polygon_mesh_processing/index.html).
Introduction and self-intersection/side-of-mesh sections: combinatorial manifold
assumptions do not exclude geometric self-intersections; closed-mesh parity has
explicit input assumptions. This is an implementation reference, not a dependency
or a certification of Rheon's convex-solid policy.

[A2] CGAL developers. Fast Intersection and Distance Computation (AABB Tree),
official manual, release 6.2.1.
[Manual](https://doc.cgal.org/latest/AABB_tree/index.html).
Intersection/nearest-hit and degenerate-primitive behavior inspected. A tree's
`any_intersection` and a nearest ray `first_intersection` have different contracts.
Rheon implements its own BVH and finite-tolerance predicates.

[A3] Tyson Brochu, Essex Edwards and Robert Bridson. Efficient Geometrically
Exact Continuous Collision Detection. ACM Transactions on Graphics 31(4),
article 96, 2012. [Author paper](https://www.cs.ubc.ca/~rbridson/docs/brochu-siggraph2012-ccd.pdf).
Introduction and CCD predicate/degeneracy construction inspected. Robust
continuous intersection requires specified trajectories and arithmetic; no
equivalence to Rheon's static center-segment test is asserted.

[A4] Basilisk contributors. Embedded boundaries, original maintained source.
[embed.h](https://basilisk.fr/src/embed.h). Gradient/metric and “Lifting the small
cell CFL restriction” sections inspected on the audit date. Its tracer
redistribution is not a ready-made Rheon momentum or mesh-water solver.

[A5] Arno Mayrhofer, Benedict D. Rogers, Damien Violeau and Martin Ferrand.
Investigation of Wall Bounded Flows Using SPH and the Unified Semi-Analytical
Wall Boundary Conditions. Computer Physics Communications, 2013,
DOI 10.1016/j.cpc.2013.07.004.
[Author manuscript](https://arxiv.org/html/1304.3692v2),
[version record](https://arxiv.org/abs/1304.3692v2). Sections 2–3 inspect
continuous versus discrete skew-adjointness. Rheon's PR121 adaptations are
explicit method variants, not a reproduction proving the source wrong.

[A6] Dan Koschier and Jan Bender. Density Maps for Improved SPH Boundary
Handling. SCA, 2017. [Author publication record](https://animation.rwth-aachen.de/publication/0554/).
Author abstract inspected: precomputed boundary-density contributions and
narrow-band interpolation. The linked 10 MB full PDF exceeded the retrieval
tool's content limit; no detailed force or reaction formula is inferred from
that abstract. Volume maps remain a related candidate named in the earlier
expansion bibliography, not a method validated by this revision.

[A7] Eran Guendelman, Andrew Selle, Frank Losasso and Ronald Fedkiw. Coupling
Water and Smoke to Thin Deformable and Rigid Shells. SIGGRAPH, 2005.
[Authors' project record](https://graphics.stanford.edu/papers/thin_shells_fluid_coupling-sig05/).
Author abstract inspected: ray-based visibility augments fluid operations and
incompressibility near infinitesimal shells. This supports that conceptual
distinction only; full PDF retrieval was unavailable in this audit.

[A8] K. M. T. Kleefsman, G. Fekken, A. E. P. Veldman, B. Iwanowski and
B. Buchner. A Volume-of-Fluid Based Simulation Method for Wave Impact Problems.
Journal of Computational Physics 206(1), 363–393, 2005,
DOI 10.1016/j.jcp.2004.12.007.
[University-hosted version of record](https://pure.rug.nl/ws/portalfiles/portal/2941055/2005JCompPhysKleefsman.pdf).
Governing/boundary conditions and Section 7 dam-break obstacle/probe specification
inspected. Matching observations and uncertainties are required for validation;
Rheon's release specimen is not a reproduction of those measurements.

The core source checks additionally re-read [S13] Section 3.5's deliberate
poly6-density/spiky-pressure kernel choices. The [S3] author endpoint returned
an 18-page scanned PDF without extractable text; screenshot retrieval failed.
Its full construction was not freshly read by this audit. Projection signs
were independently derived and checked against [S7] and native source instead.
These checks do not support a blanket claim
that unlike density/pressure kernels are always a research error. The actual
Rheon compression-energy derivation and its branch limits are in Appendix G.

## Read a model variant before using it

Inspect the supplied SST-2003 limiter, then read the primary model/variant record linked from chapter 22. Keep the publication motivation, maintained equation specification and locally executed probe as distinct evidence. The 1992 report does not specify all 2003 coefficients. An inaccessible proceedings text is identified as unread. No figure, code or dataset from those publications is included.

<details><summary>Practical study proof scope</summary>

[PracticalStudy.lean](../../../proofs/Rheon/PracticalStudy.lean) states the
exact real-arithmetic premises. The pinned [kernel build and 330-declaration audit](https://github.com/MrScripty/Rheon/actions/runs/38478346517) passed; the integrated proof files match that checked source. The independent finite oracle checks the browser formulas; it does
not prove a Rust refinement, IEEE solver, continuum prediction or runtime speed.

</details>
