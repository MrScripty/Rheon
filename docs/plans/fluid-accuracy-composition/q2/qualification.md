# Q2 bounded geometry qualification

Version1/652383e is a retained failed campaign, not silently corrected evidence.
Version2/8b9c3b2 was independently reviewed/frozen before ONE separate native run.

A1: four focused contracts pass debug/release; final selected strict Clippy passes.
Contracts cover polynomial moments/flux, orientation/repeated source winding,
finite coincident boundaries/domain refusal, exact zero-axis tangent versus tiny
positive dyadic area, and nonzero pressure versus wrong-pressure traction.
No existing production source changed in Q2; Q1's full release regression613pass,
0fail,3existing asset ignores is retained and not represented as a full Q2 rerun.

A2:24native CPU cases complete, fixed finite geometry gates pass. Raw source
vertices are flushed before construction; cell/dual/shear moments, unique MAC
faces, physical source-edge pieces, per-cell flux/stress and negative pressure
ledgers are retained. Fine disk geometry/flux/force errors1.26e-5 or below,
orders~2; worst normalized local native defect1.43e-12 against1e-10. This is
polygon geometry integration, not a solution of PDE unknowns.

A3: independent positive vertical-slice integration checks all152780 unique
moment records across12source geometries (both viscosities share geometry);
maximum normalized moment discrepancy7.883e-15. All24case ledgers,75264cell rows
and5930physical pieces are reconstructed with distinct polynomial integral
formulas. Pressure and the predeclared wrong-pressure control are checked.
Independent endpoint/line and exact-rational tangent audits distinguish native
artifacts from the independent floating oracle's own presence ambiguities.
Final review.md and raw summary/tangent audit own the precise counts and limits.

**Supported:** bounded finite-error common geometry/moment/flux/stress integration
reference under its stated cases and tolerances. No public production geometry or
exact topology admission. Five positive native apertures occur on exactly
zero-measure source faces, notwithstanding tiny error within the finite budget.
This is an explicit blocker before exact connectivity/active-DOF/B/M/S/W assembly.
Return the predicate/ownership decision to bounded method selection; no clamp,
floor, further native rerun or larger solver implementation is introduced here.

**Pending:** true solid-exterior/cut topology, velocity extension, small-cell policy,
common discrete operator and boundary lift, component/gauge correspondence,
scalable backend and independent dense solve, actual native coupled work, disk/
annulus MMS, rescaled water hydrostatics and independently reconstructed physical
traction, and conservative transport selection. Original plan thresholds unchanged.
No physical experiment, GPU/native UI or new Lean correspondence claim. Existing
exact-real StokesWork proof is relevant only once actual operators satisfy its
assumptions; it is not credited as this unassembled code's scientific evidence.

Parent/repository lead reconciles shared canonical status with G1 and controls
integration/merges. Q1 and all three P1 evidence archives/worktrees remain intact.
