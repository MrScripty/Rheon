# Independent numerical implementation review

Reviewer: delegated read-only `p1_numerical_review`, distinct from implementation
author. Reviewed the actual source, primary equations and fixed screen protocol.
No reviewer edits and no physical/production acceptance are implied.

Pre-campaign findings and dispositions:

- AHA pair antisymmetry does not imply EOS work consistency; SA gamma boundary
  operators are not exactly discrete adjoints. Both now measure actual pressure
  work and pair momentum defects; no global conservation theorem claimed.
- SA printed viscosity with dynamic mu needs whole-operator division by density.
  SI adaptation is explicit; native wall slope comes from parcel velocity/distance,
  never the exact analytic profile derivative.
- Simultaneous forward Euler cannot stabilize undamped acoustic eigenmodes.
  Research uses declared kick/continuity/drift ordering; production acoustic
  admission remains explicitly unavailable. Shear uses the actual assembled
  explicit contraction matrix with equal physical parcel masses.
- Rejected candidate velocity/work/momentum must not enter accepted results.
  Complete candidates and ledger terms are now published only after all checks;
  refused candidate fields are separate from initial/last accepted fields.
- An energy remainder cannot be labelled temporal by subtraction alone. Native
  code independently accumulates kinetic/gravity splitting, kicked-continuity
  and nonlinear EOS chain terms, and compares their sum to measured total change.
- Fixed dummy volume in all operators is an adaptation; the inspected source
  explicitly establishes this policy in continuity. Protocol now distinguishes it.
- Added independent 3D all-pairs enumeration comparison, norm-bounded rather than
  per-axis perturbation, zero-gravity helper checks and empty wall-support handling.
- Added actual nonzero free-slip campaign plus a focused fixture regression.
  Initial zero slip data would have made the comparison vacuous. Traction is
  measured, final physical time and shortened final dt are retained.
- Thick slabs cannot nonvacuously test cross-wall kernel masking. Thin geometry
  diagnostic queries now have positive unmasked kernels and exact zero masked
  density/pressure/viscosity factors, while volumetric support admission is a
  separate refusal. This is not native mesh-wall qualification.
- All evidence writers now explicitly flush and propagate write failures.

Supporting pre-campaign checks: 11 focused tests passed, strict affected Clippy
passed, format/diff checks passed. These check code and experimental contracts;
they do not decide the water accuracy thresholds.

Final campaign and result review disposition will be added after evaluation.
