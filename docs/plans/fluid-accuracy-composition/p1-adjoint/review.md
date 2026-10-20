# Independent review record

Pre-freeze review: separate read-only agent /root/p1_numerical_review inspected
the derivation, primary references, protocol, complete model/driver and evaluator.
No numerical experiment or tests had run. It verified the full gamma Jacobian,
independent pair-force expression, wall signs, frozen offsets, explicit source
mass conventions and clipped-face derivatives. Findings corrected before freeze:
scalar gamma extraction; fixed dimensional normalization; uniform hydro fields
unavailable; individual wall loads; support-boundary bands; wet-branch assertions;
reference pressure-null distinction and static scope. Final disposition: no
remaining blocker to source/protocol freeze and the declared static experiment.

The reviewer did not edit files. This is independent reasoning/code review, not
an independent physical experiment or external solver validation. Post-run field
review is recorded below.

Post-run review: independently reconstructed all16 cases/768 sites using fresh
Python formulas and Decimal45 source-profile integrals. Retained script
`independent-review/reconstruct.py` SHA256
8b6f5f338ba062247f4252fda4b2bbff46c3e931e5b52716684ce0b65913f646;
summary SHA25681035ca5d968c346818fc6439d4aaca5325740d45b96c14dfd225aa8aec1cfc6.
Maximum differences: masses8.53e-14 kg/m²; Jacobian-assembled versus native
pair/face force6.37e-12 Pa; density rates5.12e-13 kg/(m³s); volume gradients5e-16;
wall loads2.73e-12 Pa. Linear-pressure diagnostics agree to6.81e-14 relative;
alternating-pressure force to6.4e-13 Pa. All48 recorded differences meet frozen
scales. Root replay of the retained independent script reproduces its summary
byte for byte. This is a separate implementation/review, not physical validation.

The reviewer separately explained the fixed first-gradient moment728/729 and
checked >2R post-run masks, keeping them distinct from the frozen R-band gate.
The closed column's1.06105487% compression was verified and explicitly excluded
from production-envelope acceptance. It reviewed the final causal conclusion:
work repair succeeds algebraically but does not solve E0/traction; the exact
partition reference still fails hydro/boundary/null requirements. No further
native evaluation is warranted in this slice. Positive shared partition is only
a next candidate to derive, including first moments, gravity, boundary degrees,
translation and acoustic spectrum; it is not a demonstrated repair.
