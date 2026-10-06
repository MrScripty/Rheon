# One fixed public call under the new 66 KiB protocol

The unchanged frozen Rust binary c37e2345ee5bae08c594f906bdb49dd3edbce37e46baab99a9f10f761a64af6f
ran exactly its specified research_public_candidate_and_controls test. The
native test passed in 7.78 seconds. No binary/source rebuild, parameter search,
trajectory continuation, or production adoption occurred.

## Separate resource result

The fresh instruction audit reproduces the 65,552-byte aligned additional
bound. The NEW 67,584-byte resource allowance passes with 2,032 bytes remaining.
The OLD 65,536-byte protocol still fails by 16 bytes; its E1 classification
remains UNEXECUTED in the preserved historical packet. The binary's legacy
65,536-byte test counter is supplemented by an explicit external reservation
of 2,048 bytes. The complete new single-owner reservation is 542,352 bytes.
This is not a change to production memory settings.

## Numerical result

The original native prefix was recreated and verified bitwise, and its report
exactly matches the frozen baseline. One isolated cloned owner then completed
the fixed pressure/nonconstant/forward call at h=0.00078125 and acceleration
(0.0625,-0.125,0.03125), publishing version 1 to version 2 at time 0.0015625.

There were **four Newton checks, three corrections and 22 equation calls**.
Main check norms were 0.02531284307629951, 5.070578299755557e-10,
1.0251636964109857e-13 and 5.719096375827638e-14. The final fine planar report
norm is 5.718668950928878e-14; direct planar norm 6.510498011949205e-14.
Third-component finite/direct norms are 6.234267138578429e-14 and
7.258244958832171e-14. The 1e-13 thresholds and controller budgets are unchanged.

The report records mass 3.375 before/after, full constraints
2.385326094631046e-15, quadrature error 1.6940658945086007e-20, GCL maximum
1.0197366124523478e-16. Planar, third and combined ledger errors are
7.524363077049401e-17, -3.562959389330489e-17 and 6.73831650199741e-17,
respectively, within their original allowances. All ordinary third assembly,
combined work and BeforePublish gates were reached. outcome.json preserves
the full report and accepted velocity/pressure/geometry/mass/clock/stamp bits.

All eleven reached cancellation barriers returned Cancelled, preserved the
entire accepted snapshot, and reproduced the full successful report and
snapshot on retry from the same prefix. Index 11 is unused and explicitly
NOT_REACHED_BY_FIXED_CALL. The original owner remained unchanged throughout.
The frozen test compares integer bit snapshots, including signed zeros; no
decimal tolerance substitutes for state equality.

replay.svg renders the four recorded main Newton norms and the accepted mesh
with its stored third velocity component. It reads only outcome.json, does
not advance an owner, and does not reconstruct a liquid surface.

## Scope and evidence limits

This is one fixed public-call research qualification, using private ephemeral
106-bit chart workspace. The stored endpoint is authoritative; no persistent
compensation, checkpoint/restart feature or retained tail was introduced.
This does not establish trajectory robustness, production adoption, a full
3D liquid/material simulator, adhesion/density treatment, or surface
reconstruction. Original Jacobi fixtures, arithmetic refusals, failed old
memory certificate and numerical/Lean limitations remain byte-exact.

run.py is frozen before execution at 94818f15e7d9148a079a4797392a8b5e356fd59b.
It records memory-authorization.json and candidate-command-before.json before
launch, then the actual native stdout/stderr and completion receipt. analyze.py
and verify.py only replay recorded evidence and static instructions; they
never rerun the candidate or controls. Closed Git/content receipts and normal/
optimized verification bind the full unchanged inventory and actual binary.
