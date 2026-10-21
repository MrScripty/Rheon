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


## M3 selected algorithms and extension review

The same read-only reviewer checked the current P1 solid/gas models, gas energy/stress/tangent, positive-pressure wall reactions, exact element integration, dyadic grid/DOF/mass arithmetic, acoustic signs and backward-Euler work identity. It checked nondimensional merit/fallback, LU/equilibration/pivot/linear-defect policies, iteration/backtrack caps, element ordering, accepted/candidate publication and finite-domain versus nonfinite-arithmetic failure outcomes for missing selections/internal consistency. No remaining mathematical blocker was identified. This is reasoning review of pseudocode, not a termination/convergence proof or executed implementation.

Corrections from review: restrict n to2/4 and derive actual reference measures, fix element IDs; call fallback a descent direction, handle nonfinite q; retain support reactions in the full thermal balance while proving their zero work through stationary constraints; distinguish coupling-force λ from EOS pressure; label mixed/viscous verification items as extensions; and distinguish finite-domain line-search skips from terminal invalid arithmetic.

The reviewer independently checked continuous ALE weak transport signs, triangle capillary gradient/tangent and sphere-pressure sign, plane penalty and fixed-map inviscid normal FSI blocks. Thermal semidiscrete cancellation is conditional, with no fully discrete entropy theorem. It independently read allowed Laghos v3.1 cpp lines985–1018 and matched blob2a04477bca658a5fa7731691a505fc6df25dc3f8; the source's sampled-energy floor is not adopted. Denied sources were not retried. No file edits, numerical/scientific workloads, native/hardware runs or machine proofs were performed by the reviewer.

Final cross-reference review confirmed the restored segment-integral injectivity/determinant/convex-path proof and projection-contraction onto-cube argument in current Chapter36. It independently confirmed13,453 chapter words, removed stale references and retained exact-real/rounded-diagnostic limits; no blocker remained.


## M4 publication constraints and source filter

The same independent read-only reviewer compared all647 inline/display TeX expressions in chapters30–36 withf67ffdc and confirmed exact mathematical retention, preserved conditional proofs and zero fenced pseudocode blocks. The two former code blocks are practical prose. Excluded implementation sources are absent from chapter authorities; MFEM/Laghos pins, license boundaries and limited architecture claims remain. The reviewer found no mathematical or source-selection blocker and no introduced numerical/machine-proof claim.

The reviewer confirmed the build's automatic HTML discovery and its explicit implementation-guide print list: all seven chapter bodies enter print/PDF, while the new audit is an HTML/separate Markdown guide whose print links resolve to repository sources. No workflow/builder edit is needed for the website. Review was read-only, with no URL retries, edits or scientific workloads. The final source pointer wording was narrowed to the Sierra chapters actually recorded as read, and FEBio's MIT license was distinguished from its unresolved referenced-notice lookup.
