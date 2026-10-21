# Independent mathematical and source review

Reviewer: separate read-only `fem_math_source_review` agent, 2026-10-21, requested explicitly by the user. No duplicate implementation worker, external message, numerical/native/scientific run or file mutation was performed by the reviewer. This is an independent reasoning/source review, not machine-checked mathematics.

Reviewed: new chapters30–36, source map and review guide. Independently fetched source spot-checks included DOLFINx v0.9.0 energy/forms, deal.II v9.6.2 mixed condensation, MFEM v4.7 mass/hyperelastic/damping/Newton separation, Laghos v3.1 compressible model, FEBio v4.7 domain/material/geometric-stiffness separation and all six project license authorities.

## Findings and dispositions

1. **Corrected:** the first volume counterexample used a forward determinant while referring to endpoint backward Euler. Chapter34 now derives endpoint `F_new=(I-h L_endpoint)^(-1)F_old`, so a trace-zero diagonal endpoint gradient at `ha=1/2` gives `J=4/3`; forward old-geometry motion is separately labelled `J=3/4`.
2. **Added:** chapter31 explicitly distinguishes material-coordinate Lagrangian descriptions from variational mechanics, with action, endpoint variations and the semidiscrete Euler–Lagrange equation.
3. **Resolved:** curved P2 geometry required a concrete sufficient feasibility procedure. Chapter36 uses a deliberately restrictive convex-reference-domain bound `||grad_X d||F<theta<1`, with whole-element affine-gradient vertex bounds and an injectivity/path proof. No sample-only positivity or floating-point certificate is claimed.
4. **Corrected:** fluid pseudocode explicitly initializes a converged flag and rejects after the Newton cap if no converged solution exists.
5. **Qualified:** exact density-weighted L2 remap preserves angular momentum when rigid-rotation tests are present, domain/metric agree and integration is exact. Chapter35 now states this conditional fact instead of an unqualified loss warning.
6. **Editorial:** source-map provisional introduction updated; obsolete TeX `rm` syntax replaced with portable `mathrm` after Pandoc reported conversion warnings. These do not change equations.

The reviewer checked stress/tangent indices, P1 mass/shear forces, negative P2 vertex row sums, scalar backward-Euler energy, mixed off-diagonal signs, interface adjoint power, ALE GCL, conservative remap and the conditional Céa argument. The final review reported no other blocking mathematical/source findings after the named small corrections. Review limits remain those of the source map: some original papers were available only as primary bibliographic records, and no nonlinear moving-mesh convergence, uniform inf-sup, numerical/native/hardware or new formal-proof claim is accepted.

Final reviewer follow-up confirmed both small corrections and the added hydrostatic container example: pressure/body-force signs, admissible residual cancellation and 9810 Pa bottom pressure are consistent. No remaining mathematical blocker was identified.

## M2 research-closure audit

The same independent reviewer separately inspected the new research status report and changed attribution/coverage text. No web retrieval or execution was performed in this review. The reviewer confirmed that unread-source dispositions match recorded access, that reference-body attribution is separated from checked derivations, and that conditional proofs, machine proofs, pseudocode and run validation are distinct. The compressible material-mesh fluid, ALE, capillary/contact/FSI, mixed-stability and numerical-selection omission inventory was judged appropriately explicit.

The added Newmark trapezoidal/modal argument was checked, with a requested clarification that constant mass and constrained stiffness are both symmetric positive definite and acceleration states are consistent. That qualification is now explicit. A malformed duplicated metre unit in the earlier scalar example was corrected without changing numerical values. A1–A4 are now labelled M1 manuscript acceptance; A5 covers this audit/report and A6 remains pending for full research closure. The review accepts the milestone's evidence classification, not unread full-text conclusions or a validated solver.
