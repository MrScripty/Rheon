# 16 The checked Lean companion

The foundation has 20 public theorems in three modules; the expanded edition adds thirteen in Physics.lean and nine in BoundedPhysics.lean, for 42 public theorems in five modules. The bounded module also defines six helpers; its fifteen audited declarations are not fifteen theorems. Lean 4.19.0 and mathlib revision c44e0c8ee63ca166450922a373c7409c5d26b00b are pinned, with locked transitive revisions. Hosted qualification at source head ecf97d3a943ebe30d20acc1cf01b94dc08ff96a2 compiled all modules and audited 31 declarations including generated equation and proof declarations.

Only propext, Classical.choice and Quot.sound are allowed transitive axioms. Custom assumptions, admitted proofs and native-evaluation assumptions are rejected. Negative tests inject a custom axiom and an admitted theorem and verify rejection. An exact reviewed source inventory requires review when the file surface changes. It is change control; Lean's kernel does the proof check.

## Discrete operator inventory

Adjoint_identity proves the finite transpose identity for arbitrary real matrices. Pressure_energy rewrites the quadratic form as weighted squares. Pressure_nonnegative adds nonnegative weights. Pressure_symmetric establishes bilinear symmetry.

Constant_gradient_zero and constant_pressure_nullspace assume balanced columns. They establish constants in the kernel, not that constants are the entire kernel. Disconnected components and zero weights can enlarge it.

Internal_flux_conservation proves total incidence cancellation. Conservative_update proves unweighted total preservation for its exact update. Unequal physical cell volumes require separate weighting.

Projection_residual assumes input incidence equals time step times the declared right-hand side and derives corrected incidence from residual. Exact_projection additionally assumes an exact pressure solution. Neither guarantees a solution exists or an algorithm finds it.

## Transport inventory

Blend_lower, blend_upper and blend_constant prove two-point bounds and constant preservation. Convex_sum_bounds handles nonnegative finite weights summing to one. Upwind_positive proves one-dimensional positivity for Courant number in \([0,1]\). Explicit_diffusion_positive proves one-dimensional three-point positivity for diffusion number in \([0,1/2]\).

Cfl_counterexample checks a negative upwind value outside the restriction. Interpolation_not_mass_conservative checks the three-row matrix example of Chapter 6. Both are exact rational counterexamples proved by kernel-checked arithmetic tactics.

## Indexing inventory

Flatten_in_bounds proves x-fastest natural-number indexing under coordinate bounds. Staggered_count proves the cubic face-count identity. They do not verify Rust accesses, allocator behavior or machine overflow.

## What remains unproved

No theorem establishes geometry assembly, cut-cell quadrature, positive definiteness of every configured domain, pressure uniqueness, CG convergence, trajectory accuracy, liquid-volume conservation, IEEE error bounds, Rust or GPU equivalence, physical realism, runtime performance or diffusion-image fidelity. No Navier-Stokes existence claim is made.

These boundaries make the proofs useful: exact algebra removes sign and structure ambiguity while implementation and empirical claims retain appropriate evidence. Weighted cut-cell adjointness, component-aware gauges and refinement from array kernels to abstract operators are valuable future work.

## Reproduction

From proofs, fetch the pinned mathlib cache, run lake build, then lake env lean AxiomAudit.lean and the negative audit script. The workflow checks unchanged dependency files. Historical success applies to its exact source revision. Changed proof sources require new qualification.

The expansion qualification and all new assumptions are recorded separately in Appendix F and `expansion/proof-qualification.json`. The integrated pinned project compiles all 42 public theorem statements and audits 60 declarations. The current source binding is `expansion/bounded-proof-qualification.json`; the earlier 33-theorem/45-declaration receipt is retained as historical evidence. BoundedPhysics derives planar first contact/clipping and dissipative work from finite-strain backward-Euler equations. Arbitrary-mesh queries, symmetric-gradient assembly and a coupled liquid implementation remain separate research work.
