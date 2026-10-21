---
title: "Rheon: two complete minimal Lagrangian FEM algorithms"
author: "Book author: Puma"
date: "21 October 2026"
---

# What is complete

Chapters30–36 now provide a principles-first progression and two fully selected reference algorithms: **S**, a three-dimensional hyperelastic solid, and **G**, a three-dimensional inviscid isentropic material-mesh gas. The complete algorithm is authored pseudocode with explicit equations, dataflow and bounded failure. It has not been implemented or executed in Rheon. The restricted fitted-height/fitted-ALE studies remain separate research; they do not establish a general FEM implementation.

The [seven-chapter manuscript](../README.md), [Chapter36 specification](../chapters/36-reference-fem-solver.md), [source map](source-map.md) and [historical M2 audit](research-status-report.md) are the review artifacts. Checkpoints `9489aa3c1bd6652f9ab1874fce904853b9063dc3` and `5b5b1de1e6aa3463c7214243b62538a637240e7a` remain ancestors. This M3 supplement supersedes their missing minimal-fluid/numerical-selection inventory, while retaining their source-access limitations. Puma's credit, old research, original local work, product PR145/146 and scientific admission files are preserved.

# The learning progression

1. Chapter30 starts with material/spatial coordinates, connected meshes, the motion map, F/J and material conservation.
2. Chapter31 connects stress measures, objectivity, constitutive models, strong/weak forms, variational mechanics and total/updated descriptions.
3. Chapter32 constructs bases, quadrature, mass, forces and consistent tangent assembly.
4. Chapter33 connects explicit/implicit integration, nonlinear residuals, line search, Krylov/direct solution and conditional stability.
5. Chapter34 distinguishes mixed incompressibility from a compressible material-mesh fluid. It derives gas thermodynamics, energy, stress and tangent before discussing thermal/viscous extensions.
6. Chapter35 derives contact/interface work, ALE transport and conservative transfer, then concrete capillary and coupling derivatives. Particle/grid methods retain distinct geometry/state meanings.
7. Chapter36 closes the model, mesh, quadrature, boundary, solver, arithmetic and failure choices for S/G, with complete construction/assembly/step/publication pseudocode and hand checks.

# Fixed minimal algorithms

Both use the unit reference cube, constant reference density1, continuous P1 motion/velocity on affine tetrahedra, exact consistent P1 mass and one barycentric point for the homogeneous constant-F element energy/force/tangent. The structured grid has n=2 or4, default2, six prescribed Freudenthal tetrahedra per cell and explicitly ordered node/cube/element IDs. Actual dyadic reference geometry determines volumes/gradients; it is not replaced with nominal measures to hide a mismatch.

S uses the specified logarithmic neo-Hookean energy with mu=lambda=1 and a full clamp on x=0, zero natural traction elsewhere. G uses gamma=5/3, p0=rho0=1, pressure p=J^(-5/3) and reference internal energy W=(3/2)J^(-2/3). Coordinate walls constrain only their normal motion. Gas pressure is EOS-derived, with nonzero wall reactions, no independent pressure DOF or arbitrary gauge. G has no elastic shear stiffness, physical viscosity, heat equation or shock-admissibility guarantee.

The stated SI scales nondimensionalize all arrays before merit minimization. Initial velocity is the nodal bubble specified in Chapter36, amplitude |U|<=1/16, default1/16. Use backward Euler, h=1/128, eight steps. A candidate starts from the accepted displacement. Every residual uses the same accepted previous displacement/velocity; the tangent differentiates that residual. Reduced free equations retain consistent inertia; constrained residual entries provide support reactions.

Dense partially pivoted LU uses the declared left row equilibration, pivot policy and recomputed linear-defect check. A finite residual/Jacobian permits a specified residual-merit gradient fallback when the direct attempt is unusable. Directions have a fixed norm cap; Armijo examines exactly20 candidate step lengths, with at most32 accepted nonlinear updates. Chapter36 specifies every numerical gate and constant, with no assertion that they have been tuned or certified. No changes to Rheon's validation thresholds follow.

Finite geometry-domain violations skip a line-search candidate. Nonfinite geometry/model/residual/merit arithmetic is terminal; it is not hidden by continued backtracking. Unusable direct algebra has only the expressly defined fallback. Failed convergence, stagnation, exhaustion, memory/input violations or cancellation return the accepted prefix. Accepted/candidate buffers and a final cancellation barrier prevent partial publication. The sufficient gradient geometry bound is a conditional exact-real argument; its computed binary64 gate is not a rigorous rounding certificate.

# Fluid and extension derivations added

For a smooth adiabatic ideal gas, de=p d(rho)/rho² and p=(gamma−1)rho e give rho=rho0/J, e=e0 J^(1−gamma), p=p0 J^(−gamma). Differentiating reference energy gives P=−pJ F^(-T) and its full consistent tangent. No unread article theorem is required for these independently checked calculations. The tangent, compression values and wall-rest response are in Chapter34.

The next thermal semidiscretization uses P1 velocity and cellwise constant specific energy, with pressure work paired with momentum and stationary support reactions retained. Dotting the constrained momentum equation and summing energy proves conditional semidiscrete power cancellation. Viscous heating is nonnegative for the stated Newtonian coefficients. This is not a fully discrete entropy/shock/heat solver.

Chapter35 independently derives moving-test ALE weak mass, momentum and total-energy fluxes. It explains why mesh smoothing needs a discrete GCL/transport choice and why an isentropic state cannot absorb remap kinetic loss as heat while retaining the same entropy law. Triangle surface-energy gradients and tangent actions, a spherical Laplace-pressure sign check, a fixed-plane penalty and fixed-map normal-only FSI blocks are explicit. Moving interfaces need their actual geometric derivatives; contact topology, nonmatching map construction and a stable coupled solve remain outside S/G.

# Checked mathematics and remaining qualification

Independent read-only review checked symbolic derivatives, signs, constraints, small arithmetic and missing algorithm steps. The review identified and corrected: pressure-wall forces in thermal power balance; non-dyadic mesh ambiguity; element ordering; nonfinite/backtracking policy; gradient-fallback scaling/descent; and the distinction between EOS pressure and coupling multipliers.

Hand checks include default48 tetrahedra/27 nodes, free scalar counts S54/G27, element mass diagonal1/480 and off-diagonal1/960, centre mass1/20, bubble kinetic energy1/10240 J, solid rest/shear, gas wall rest/compression/breathing tangent, linear acoustic signs and the exact backward-Euler work-defect identity. No numerical result is inferred from these calculations. The gas energy is convex in J, which does not establish global nodal convexity or an unconditional nonlinear time-step energy theorem.

These are standard derivations, original exposition and conditional mathematical proofs. They are **not machine-checked proofs, numerical solver validation, native verification or hardware evidence**. No scientific/full-book build, model download or experiment ran. Only document parsing, links, equation syntax, direct rendering and Git scope checks are admitted here.

The minimal S/G exposition closes the requested end-to-end reference path. Advanced research still needs: actual mixed-family stability/consistency; a fully discrete thermal/shock model; selected ALE flux/time/GCL/remap algorithms; robust distributed contact/moving FSI; and any claimed spatial/time convergence. Future implementation and bounded numerical qualification need separate admission. Selected S/G are deliberately bounded teaching models, not arbitrary general3D/contact/FSI software.

# Primary sources, access and licenses

The [ordinary-text M2 report](research-status-report.md) lists all13 paper/report entries, six pinned project/license studies, exact failed targets and claim-by-claim dispositions. Several full bodies remain unread. No denied target was retried or accessed through another route. Bibliographic provenance is separated from the checked derivations above; IPC guarantees remain author-reported, and the original MPM algorithm is not claimed source-audited.

The only new M3 primary retrieval was previously accessible [Laghos v3.1 laghos_solver.cpp](https://github.com/CEED/Laghos/blob/v3.1/laghos_solver.cpp), blob `2a04477bca658a5fa7731691a505fc6df25dc3f8`. Independently reviewed lines985–1018, especially1005–1008, substantiate its sampled-energy, ideal-gas pressure/sound-speed implementation. Its energy floor is not imported. The inspected [LICENSE](https://github.com/CEED/Laghos/blob/v3.1/LICENSE) is BSD2-Clause with NOTICE obligations; no third-party source was copied. The Dobrev2012 paper body remains unread and is not a proof source for our algorithm.

# Delivery and publication boundary

M2's seven-page PDF was saved successfully to Library as `libfile_4209b1326e6481919cd7beb90d6259c1`, backing file `file_00000000af3481f5a2302ae92a343df8`. It is an intentionally preserved checkpoint report. The current supplement and solver preview are separate deliverables. Parent coordinates review, publication/normal CI, merging and deployment. No push, merge or deployment has occurred.
