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

Final pre-campaign disposition: adequate to freeze and execute this bounded
research campaign; no production method admitted. Frozen source checkpoint:
39098e0e0b0b0f8c966ff7f811d548b6d396ec15.

## Independent campaign disposition

**Return both adaptations to method selection.** Reviewer independently parsed
all 108 hydro, 162 shear and 36 cloud summaries and raw field files. Every
hydro density/pressure RMS and every shear velocity RMS agrees to roundoff;
pressure matches the EOS; all accepted centers remain above the bottom; every
reduced trajectory reaches .1 s. Direct total-energy reconstruction agrees
within 8.46e-11 J/m² using an independently evaluated log expression. The
accumulated ledger closure <=7.25e-12 J/m² verifies accounting, not conservative
dynamics: integrated pressure-work defects are orders of magnitude larger than
the fixed characteristic energy target.

Verified decisive failures: perturbed AHA n32/h1.5 speed .167893 m/s and
pressure 19.0076%; ordered SA same configuration .062635 m/s and 4.64473%; SA
fine traction 29.33–35.87%, Poiseuille momentum defect 2.93e-4–3.38e-4 kg/(m s),
and multiple spatial orders below .9. Fourfold diagnostic dt reduction barely
changes these failures. All 36 raw cloud fields reproduce rates, moments and
power to summation roundoff; powers are negative and momentum rates negligible.
Wendland's fine instantaneous rate success does not close consistency:
fixed-support partition/Laplacian moment biases persist under refinement.
Nonzero free-slip invariance passes.

The report's counts, values and capability limitations were independently
reviewed. Two wording findings were corrected: passing density targets does not
establish improvement against the preview, and a future justified wall-reaction
derivation remains possible even though post-hoc relabelling of current defects
is invalid. Final disposition: the necessary-screen implementation and retained
negative information are reviewable; P1 production remains unaccepted.

Recommended next mathematical work: first establish constant/linear-field
consistency and neighbor growth; derive continuity, pressure work and wall
reaction from one discrete construction; derive viscosity/traction from the
same quadrature and boundary lift; repeat focused algebra/moment/traction gates
before another trajectory campaign. No independent Lean kernel rerun, native
GUI, GPU, physical experiment or full production water test is claimed.

## Ordinary test discovery

The reviewer checked the small integration adapter. It only makes
the existing eleven checks discoverable by ordinary Cargo test; it does not
execute the campaign CLI or change its numerical sources. All example sources
remain byte-identical to 39098e0. Scientific review/disposition is unchanged.
The complete 582-test existing CPU release suite and 11 adapter checks passed;
the partial debug run is explicitly retained as interrupted, not accepted.
