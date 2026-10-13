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

## Reading the newer implementation bridges

The counts and revision identities above describe retained historical
qualifications. They do not automatically include modules added for the
current material preview or adaptive CPU scheduler. Each extension needs its
own reviewed source inventory, normal pinned Lean build and allowed-axiom
audit. A linked theorem statement, a native regression test and a fresh
qualification receipt answer different questions.

For a first example, two particles exchange equal and opposite forces. In
exact arithmetic that cancels their internal contribution to total momentum.
It does not cancel gravity or a wall impulse, and it does not show that a
parallel neighbor loop evaluated each intended pair. Read the assumptions,
then locate the code and the independent test that connect the statement to
the implementation.

## Checked material-preview proof bridge

[`Rheon.MaterialPreview`](../../../proofs/Rheon/MaterialPreview.lean) isolates
several small contracts: nonnegative liquid pressure; reference ideal-gas
pressure; symmetric pressure-pair coefficients; pair action/reaction; viscous
work, momentum and a conditional finite-step energy bound; nozzle-profile
bounds; the prescribed pressure-head identity; and a reservoir mass ledger.
The [material guide](../implementation/material-preview-contracts.md#proof-and-execution-boundaries)
maps these statements to the actual formulas.

For an isolated viscous pair, nonpositive power is weaker than energy decrease
after an explicit step. The latter also needs a bound on timestep, drag and
inverse masses. The theorem expresses that missing condition; it does not
certify the complete adaptive controller. No statement here proves the
compression-energy derivative through the native density floor, multiphase
interface accuracy, surface curvature, gas choking, phase change or physical
calibration. Nor does a mass-ledger identity prove that source credits survive
cancellation in Rust: that is a state-publication test.

## Checked CPU-scheduling proof bridge

[`Rheon.CpuScheduling`](../../../proofs/Rheon/CpuScheduling.lean) describes
bounded integer work and dependency relationships. It provides a mathematical
bridge for the [scheduling implementation](../implementation/cpu-scheduling-contracts.md),
whose actual scoped pool, thread-local ownership and callbacks remain Rust
execution concerns. Coverage of a finite set is not a proof of race freedom
for arbitrary callbacks, and ready-wave ordering is not a throughput bound.
`chunk_covers` and `chunk_unique` concern half-open intervals;
`same_wave_has_no_dependency` concerns the declared nearest-axis predecessor
relation. `successful_trace_explores` assumes a stable workload, serial
successful invocations and exactly one counter record per invocation. It is
not a fairness claim for overlapping or failed work.

The practical reading rule is the same as for pressure: keep the proven
algebra, native execution evidence and measured performance distinct. A change
to a source module invalidates an older source-bound qualification; it must
not be hidden by reusing a theorem count or a successful historical screenshot.

## Inspect the premise before the result

The pressure-proposal example derives residual (e, −e) from a specific two-cell
operator. A checked condition |e| ≤ t bounds both physical divergence components
because both volumes are one. It does not establish zero divergence when t > 0.
At e = 1/16 and t = 1/8 the condition passes with nonzero error; the Lean witness
states this explicitly. A gauge shift leaves this particular pressure jump
unchanged, while an error in that jump changes the corrected flux.

The browser is neither a proof assistant nor a general pressure backend.
Read the theorem's operator, volumes and tolerance premises before using it
for another mesh. Native source/receipt admission, floating-point refinement,
model validation and runtime performance require their own evidence.

<details><summary>Practical study proof scope</summary>

[PracticalStudy.lean](../../../proofs/Rheon/PracticalStudy.lean) states the
exact real-arithmetic premises. Kernel qualification for these new statements
is pending. The independent finite oracle checks the browser formulas; it does
not prove a Rust refinement, IEEE solver, continuum prediction or runtime speed.

</details>
