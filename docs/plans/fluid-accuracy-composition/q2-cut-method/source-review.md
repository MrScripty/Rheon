# Independent pre-execution source review

Numerical reviewer `/root/p1_numerical_review` inspected checkpoint fee53b2
without running selected tests, native fixtures or numerical oracle evaluation.
The review found no mathematical blocker in M/B/K factors, symmetric-strain
engineering-shear weighting, stress/pressure signs, common partition, active
basis support, or physical/Cartesian trace ownership. Full consistent mass and
mu weighting of engineering shear were specifically checked. Every cell's B
includes Cartesian flux and physical source-boundary trace.

The review preserves builder-only scope: no pressure solve, cut-uniform inf-sup,
conditioning, timestep or physical water admission follows from source inspection.
Primary-source research and method/protocol review preceded this implementation;
the actual native and independent deciding evidence remains pending.

The final source delta uses existing GridGeometry face_counts/face_index/
face_position directly for X/Y lattice nodes. No native coordinate arithmetic or
active-set criterion changes. The reviewer confirmed the delta against src/geometry.rs: X/Y counts, x-fastest
face IDs and XY positions match the reviewed lattice, with z=0 bookkeeping
unchanged. No source/protocol blocker remains to CV1 source freeze and focused
contracts. This is prospective experiment admission, not numerical acceptance.
Strict Clippy compile-only passes. The fixed campaign waits for oracle freeze.

The separately authored independent Fraction/polynomial runner is the declared
always-on analyzer. It is retained with the external evidence archive, not an
absent repository script. Its sources must be frozen before the fixed campaign
is inspected/evaluated. The qualification record must name exact source hashes,
command recipes and results for normal Python and -O/-OO. No assert statement
may serve as an acceptance guard.

Before the first campaign, driver review found that early return could omit
later immutable fixtures. The final driver now retains per-case refusal.txt,
continues all four cases and exits nonzero if any fail. Reviewer caught/corrected
the refusal CSV row to five fields and confirmed the final source. No numerical
operator, fixture or gate changed. Preliminary 5d352e0 contract/build receipts
remain separately retained; no campaign ran from that checkpoint. Final source
freeze and focused checks bind the corrected driver. The prospective analyzer's
summary-success literal was corrected to the native `passed` before any data.
