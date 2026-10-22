# CV2 deciding qualification

Disposition: CV2 fixed24case static coupled-solve reference ACCEPTED. All native,
independent R1/R2/analytic/physical and45interpreter/tooling checks pass. No production/PDE/evolving-surface admission.
No essential physical/user input is missing for the synthetic problems.

Numerical source b0a9d52cc266cb651c5e67c07ec5f4f82419db55.
Binary SHA256 c1f115f54e2867a2429b6d04305240ab242a1745b4c249e1e18c5512d3babfde.
Prospective independent oracle freeze SHA256:
5382622591188efb6d9148077f8940d4269ac7717424631423b18169b47e0705.
The ONE fresh release CPU campaign started04:23:38UTC, exited0, and retained
all24steps and all baseline/selected0..3states. No solver fallback/iteration
selection or criteria/source change followed data. Existing CV1 builder/export
modules were reused read-only; all65 operator CSV files match prior evidence
byte-for-byte. The complete native corpus has178CSVfiles.

| Fixture | Six native solves (dt.125/1 × unforced/rigid/affine) | Maximum fixed discrepancy |
|---|---|---:|
| rectangle | all pass | 2.109423746787797e-15 |
| square-17 | all pass | 1.861288900784075e-12 |
| disk-64 | all pass | 1.121325254871408e-14 |
| corner-triangle | all pass | 1.992850329202156e-13 |

Native fixed budget1e-10. Operator/factor shapes, original momentum/continuity,
physical closed-cell flux, raw/corrected/skew work, residual-work bound,
unforced dissipation/energy, boundary work, physical linear/angular momentum,
rigid and absolute2Pa affine coefficients all pass. These native checks alone
do not establish original-coordinate solution accuracy versus either reference.
All15debug/15release contracts, strictClippy, formatting, lockedreleasebuild pass.

The square's original fullmass condition6.5331614839e11 is unchanged, with no
basis/support/pressure removal or mass floor. It must be distinguished from A,
KKT and transformed factor conditioning. Independent exact-integral R2 and
exported-target R1 use80/120digit LU,160digit fresh residual checks and fixed
agreement/residual requirements. They are numerical references, not certified
interval enclosures. Sealed custody and Library identity are recorded in evidence-manifest.md.

Independent complete reconstruction in normal Python/-O/-OO: every selected-final coefficient passes
both original-coordinate references. Max normalized errors:

| Reference | Velocity/Uref | Pressure/Pref |
|---|---:|---:|
| R1 exact exported f64 target | 1.075398839959988e-16 | 1.943511817682625e-16 |
| R2 independent exact integral target | 1.861216501617988e-12 | 3.354808230044388e-13 |

The R1/R2 perturbation gaps are1.861224307747211e-12 velocity and
3.355371183394384e-13 pressure. The remaining error is principally assembly/load/
RHS representation, not linear-solver arithmetic. All are within the fixed1e-10
gates, without changing basis, coefficients, pressure rows or physical inputs.

The unscaled comparator fails velocity accuracy on all six square cases. Its
worst R1 velocity error is2.549277323943581e-9 (R2:2.551134021238696e-9), while
its fixed-scale original-equation residual is only9.92930057184e-12 and passes
the residual budget. This is retained negative evidence, not a fallback method.
Selected iteration0 already has R1 velocity error<=8.738361221325274e-14;
correction1 reaches final R1 roundoff. All3corrections remain mandatory; the
protocol did not choose an iteration after observing reference accuracy.

For square dt.125/1, unscaled A condition is5.5613665e7/5.2426437e7 and scaledA
condition9.8951541/73.9158428; weighted-divergence Z condition1.3900510/1.4860569.
These are finite SVD diagnostics, not rigorous enclosures or universal cut bounds.
The preserved CV1 condM6.533161484e11 came from exact-integral-rounded M; the
current nativef64M diagnostic is6.533173871e11. The65 native operator CSV files
are identical: this difference does not indicate changed geometry/support/mass.

Reference80/120digit agreement<=2.634e-78 and direct residual<=1.171e-80 at80digits
(<=1.412e-120 at120digits), far within prospective1e-40/1e-60 requirements.
No numerical-reference certificate or PDE-convergence inference is made.

All45 frozen independent tooling checks pass: complete normal/-O/-OO reports
are byte-identical and all24scientific cases pass;13malformed fixtures in each
mode refuse at their specifically declared guard. One deliberately wrong final
velocity coefficient with UPDATED internally consistent residuals is tested in
each mode; all three runs retain full24case analyses and fail actual R1 velocity
accuracy. Thus neither parser consistency nor a reported residual substitutes
for solution accuracy. All178native CSV hashes remain unchanged after evaluation.

CV2 demonstrates an accurate bounded dense solve for these fixed source geometries,
dt values, physical loads and fields, with actual absolute-pressure patch recovery
and fullmass work/momentum/flux consistency. It does not establish generic cut
inf-sup/conditioning, finer-mesh accuracy, hydrostatics/MMS convergence, moving
geometry/transport/GCL, solid Dirichlet lifting or production water. No essential
synthetic input remains unresolved. Root source admission/integration is owned
by the repository lead; this artifact authorizes no merge or cleanup.
