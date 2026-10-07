# Archived baseline pressure-complement attribution

This separate packet follows the retained-cache replay repair result
`c7003d35b403a83ae329035ef37eeb3df7ed7a78`. It uses only the retained case41
baseline fields and original fixture; no new equation, point, Newton controller,
correction or owner is invoked. Analysis/source precheck:
`14e132f797fb24b0342f9edfa43ce9848fdb42ff`, tree
`11326e89bb6b6055029b47d5f78f881c6d8baf71`.

Original source/preflight/execution/result remain respectively
`8cc37c4473c723c3436e6be5d859af90695c2c37`,
`fb710371326d552fe91bafff2995340758d14ed0`,
`c2c583180f23316c6ac2ac89da434bf313121980`, and
`455af0e35c25037d2ff904bb92fd71b5d6b1f5c9`.
Raw stdout SHA-256 remains
`8a7ec54e86e0f6ec4686bc5eeca93856e54a1b8a7972eafc82cc295061371287`;
ELF SHA-256 remains
`c483c840f34d60c58eeb01fbaeec7e745edf817e84ab02655e44b7e2900a3a07`.
This analysis does not need or execute the ELF. Its own source-bound checks read
committed source/raw-result inventories, not ephemeral caches. It neither
amends nor claims completeness for the historical cache-pinning manifest.

## What already existed

The original packet independently supports exact ranks A=6, B=16, J=22;
conditions raw 2109.07337, dual-mass 2196.31378, complementary acceleration
3.82738. No missing acceleration direction or rank deficiency is supported.
The poorly conditioned full mixed system and the dominant complementary
residual are different observations. No new rank/SVD computation is done here.
The separate projection result `def4ec2238405155c728c98cb1a76929723b9a01`
reports predicted scaled displacement 2.390532565550451e-13 and coordinate-lattice
linear residual 4.2915669718609846e-15, without a nonlinear step.

## New stored-endpoint attribution

All norms/projections below use the unchanged captured M and
W=M^-1/(27/8), pressure projector P=B(B^T W B)^-1 B^T W and complement C=I-P.
Here C denotes the projector complement, not the coordinate scaling matrix in
the projection packet. Exact rational identities and 80/120-digit normal/-O
displays agree.

Hold the stored binary D fixed. First reconstruct its exact selected fifteen-row
chart using the stored seven known endpoint values. Then reconstruct that same
chart using the exact binary-rational formula eta0+h*alpha instead of the stored
rounded eta1. All accepted/start authority, masses, embedding, force, donor,
geometry and pressure fields remain frozen. Split exact stored residual as:

r_stored = (M/h)(z_stored-z_chart_known_stored)
         + (M/h)(z_chart_known_stored-z_chart_known_ideal)
         + r_frozen_ideal_endpoint.

This is an exact frozen-inertia attribution, not a new physical endpoint.
Selected rows are enforced by an external rational diagnostic only; full
unselected constraints and the moving chart/nonlinear fields are not repaired.

| Contribution | Raw force norm | W pressure-complement norm | Signed complementary alignment |
|---|---:|---:|---:|
| Exact stored stable residual | 1.0688253531784157e-13 | 9.861378297182208e-14 | 1 |
| Dependent selected-chart rounding inertia | 6.498663179170984e-14 | 6.833762361443958e-14 | 0.2481944084818046 |
| Known eta endpoint rounding inertia | 6.871785672061165e-14 | 7.492721601499891e-14 | -0.050747690954505154 |
| Frozen fields with ideal known selected-chart inertia | 8.188532000676303e-14 | 8.030545684894771e-14 | 0.8025532824727005 |
| Native stable accumulation minus exact stored replay | 8.98769663910027e-18 | 5.652255921999323e-18 | 2.9716997789060154e-5 |

Signed alignment is dot(C r_stored,C contribution)_W / ||C r_stored||_W^2.
The three endpoint contributions sum exactly to 1 in this algebra. These are
not independent percentages or physical error shares: negative alignment and
large cross terms expose cancellation. Removing the dependent-chart term alone
leaves raw norm 1.0476852236284189e-13, still above the unchanged gate. Removing
both endpoint inertia terms leaves a frozen remainder below 1e-13; the exact
squared comparison is archived. Other fields would change at a real endpoint,
so this cannot qualify a candidate, explain the full refusal or justify relaxing
a threshold. Most complementary alignment remains in the frozen remainder.

## Actual residual terms and cancellation

The script also W-projects the five actual stable residual groups and verifies
that their vectors and signed complementary projections sum exactly to the
stored residual. This adds the missing metric attribution to earlier raw-term
diagnostics; it does not repeat earlier force assembly or integrations.

| Actual retained term | Raw norm | W complementary norm |
|---|---:|---:|
| Chart increment inertia | 0.07919714233180022 | 0.07595963792108319 |
| Mass change inertia | 1.6034413726209714e-5 | 7.4162994397061195e-6 |
| Captured combined force | 0.11673948953742272 | 0.04216210682552027 |
| Body force | 0.12501506951499042 | 0.0633462411492252 |
| Endpoint donor transport | 7.651055755165332e-5 | 6.065582507425658e-5 |

Their signed alignments are of opposite signs and reach about 2.6e11 in
magnitude. The 9.86e-14 complement results from cancellation among much larger
terms. Term magnitudes alone identify neither erroneous assembly nor arithmetic
uncertainty. The tiny stable accumulation gap excludes upstream stored-state,
force and donor construction errors; it is not a true residual error bound.
The full mixed conditioning supplies no arithmetic floor, FD truncation bound
or root distance. The old FD comparison 2.5153e-10 remains a captured-field
comparison only.

## Current smallest continuation: archived data only

This note supersedes the earlier projection packet's prospective one-observation
proposal for the current authorized scope. No new observation or controller run
is proposed for execution now. The smallest useful continuation is to reconcile
this remaining complementary vector against the **already retained** compensated
chart parts and force/donor primitive captures, first verifying their exact
model/input/endpoint match to this baseline. Isolate their projected changes
entrywise with exact cross terms. Do not infer a missing primitive or recompute
an equation if archival coverage is insufficient; report that provenance/data
limit. Such a replay could distinguish which stored construction differences
are observed, but still would not certify a real-map error budget, derivative,
root basin or lower arithmetic floor.

The original Newton refusal, seven-correction budget, gates and every failed
artifact remain unchanged. The 66,368/67,584 memory certificate stays incomplete.
No main/production change or PR is made. Reproduce with `python3 -B analyze.py 80`
and `120`, with/without `-O`. Clean-checkout reproduction also disables bytecode
writes and compares outputs exactly, without restoring ephemeral cache files.
