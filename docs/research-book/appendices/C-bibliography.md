# C Primary sources

The works below provide historical and technical context. Graph definitions, worked arithmetic, recommendations, experiments and Lean contracts are independently specified for Rheon. No third-party figures or long quotations are reproduced. Publisher-only citations identify original works without implying full-paper reproduction.

[S1] Jos Stam. Stable Fluids. SIGGRAPH 1999, 121-128. DOI 10.1145/311535.311548. [Author paper](https://www.dgp.toronto.edu/public_user/stam/reality/Research/pdf/ns.pdf). Semi-Lagrangian graphics simulation.

[S2] Francis H. Harlow and J. Eddie Welch. Numerical Calculation of Time-Dependent Viscous Incompressible Flow of Fluid with Free Surface. Physics of Fluids 8, 2182-2189, 1965. DOI 10.1063/1.1761178. [Paper](https://www.cs.rpi.edu/~cutler/classes/advancedgraphics/S10/papers/harlow_welch.pdf). Marker-and-cell foundation.

[S3] Alexandre J. Chorin. Numerical Solution of the Navier-Stokes Equations. Mathematics of Computation 22, 745-762, 1968. [Author paper](https://math.berkeley.edu/~chorin/chorin68.pdf). Projection methods.

[S4] Robert Bridson and Matthias Müller-Fischer. Fluid Simulation. SIGGRAPH 2007 Course Notes. [Author notes](https://www.cs.ubc.ca/~rbridson/fluidsimulation/fluids_notes.pdf). Broad numerical implementation context.

[S5] Jonathan Richard Shewchuk. An Introduction to the Conjugate Gradient Method Without the Agonizing Pain. 1994. [Author paper](https://www.cs.cmu.edu/~quake-papers/painless-conjugate-gradient.pdf). Krylov methods and conditioning.

[S6] Andrew Selle, Ronald Fedkiw, ByungMoon Kim, Yingjie Liu and Jarek Rossignac. An Unconditionally Stable MacCormack Method. Journal of Scientific Computing 35, 350-371, 2008. [Publisher](https://doi.org/10.1007/s10915-007-9166-4). Corrected advection.

[S7] Christopher Batty, Florence Bertails and Robert Bridson. A Fast Variational Framework for Accurate Solid-Fluid Coupling. ACM Transactions on Graphics, 2007. [Author paper](https://www.cs.ubc.ca/labs/imager/tr/2007/Batty_VariationalFluids/variationalFluids.pdf). Boundary coupling.

[S8] Stanley Osher and James A. Sethian. Fronts Propagating with Curvature-Dependent Speed: Algorithms Based on Hamilton-Jacobi Formulations. Journal of Computational Physics 79, 12-49, 1988. [Author paper](https://math.berkeley.edu/~sethian/2006/Papers/sethian.osher.88.pdf). Level sets.

[S9] C. W. Hirt and B. D. Nichols. Volume of Fluid Method for the Dynamics of Free Boundaries. Journal of Computational Physics 39, 201-225, 1981. [Publisher](https://www.sciencedirect.com/science/article/pii/0021999181901455). Volume fractions.

[S10] Douglas Enright, Ronald Fedkiw, Joel Ferziger and Ian Mitchell. A Hybrid Particle Level Set Method for Improved Interface Capturing. Journal of Computational Physics 183, 83-116, 2002. [Publisher](https://www.sciencedirect.com/science/article/pii/S0021999102971664). Particle interface correction.

[S11] Christopher Batty and Robert Bridson. A Simple Finite Difference Method for Time-Dependent, Variable Coefficient Stokes Flow on Irregular Domains. 2010 preprint. [Paper](https://arxiv.org/abs/1010.2832). Coupled viscosity and pressure.

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
