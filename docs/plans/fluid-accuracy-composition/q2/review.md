# Independent Q2 geometry v2 disposition

Frozen numerical source: `8b9c3b294c63527db33c8f5faccfb24f19b4217f`.
The one native v2 campaign completed with24 case rows. Version1's exit1 and
header-only campaign remain separate and unadmitted. No candidate/source/campaign
file or criterion was edited or rerun by this reviewer; writes are confined to
this independent evidence subtree.

**The frozen finite-error geometry, moment, interval, polynomial flux and stress
integration criteria pass independent reconstruction. Exact connectivity and
active-DOF admission remain blocked by five native positive apertures on exact
zero-measure tangent faces.** This is a bounded private geometry prerequisite,
not completion of coupled Stokes, MMS, exterior-obstacle or production water.

## Independent coverage and method

Twelve distinct source geometries are checked against actual exported source
vertices, with no trigonometric reconstruction assumption. Two viscosities make
24 case rows. Source/moment/face files are byte-identical across the viscosity
pair; geometry is independently integrated once per source case and both copies
are verified. Counts below are unique geometry records unless stated otherwise:

| Association / diagnostic | Records checked |
|---|---:|
| Primal cells | 37,632 |
| X-face velocity duals | 38,192 |
| Y-face velocity duals | 38,192 |
| Vertex shear duals | 38,764 |
| All moment records, six components each | 152,780 |
| Unique native Cartesian face queries, including zero intervals | 76,384 |
| Native positive face records | 22,321 |
| Physical source-edge pieces | 2,965 unique / 5,930 across viscosities |
| Native cell-ledger rows across viscosities | 75,264 |

`reconstruct.py` integrates the convex source with a separate positive vertical
slice construction. Lower/upper source chains are piecewise affine; clipping
their ordinate interval against each primal/dual box produces positive lengths.
Two-point positive quadrature exactly integrates the resulting degree≤3 moment
polynomials in real arithmetic. This does not use native rectangle polygon
clipping or Green/shoelace moment formulas. Native X/Y MAC indices, overhanging
dual boxes and shear associations are explicitly reconstructed.

All six per-region moments (volume, mx,my,mxx,mxy,myy) are compared at the frozen
h², h²L_ref and h²L_ref² scales, numerically h² for L_ref=Href=1m. Maximum error
is 7.883e-15, below1e-10. Separate association maxima are 7.883e-15 primal,
7.106e-15 X/Y dual and 6.440e-15 shear. The oracle is an independent binary64
calculation of polynomial integrals, not a formal roundoff enclosure.

Direct source-edge/line intersections independently check all Cartesian face
intervals at scale h; maximum discrepancy is 7.106e-15. Every physical piece's
original edge ID, native cell assignment, endpoints, outward normal and first
boundary moments are reconstructed. Segment fluxes use analytic cubic/quadratic
polynomial means, independently of the native Gauss implementation. Full linear
stress is reconstructed as tau*n-p*n, alongside pressure and the p=0 control.
Every cell's signed shared-face/physical-boundary assembly and ledger are checked.

Largest differences between native diagnostics and reconstruction are 3.553e-14
for solenoidal flux/h² and 2.865e-14 for full traction/h². Direct reconstructed
closure maxima remain within all1e-10 gates: normal1.422e-14, first moment
6.721e-13, cubic2.396e-13, solenoidal1.009e-12 and traction1.432e-12. Different
polynomial evaluation orders explain rounding differences from native maxima.
The reconstructed negative-pressure control agrees with (mu,-3mu)*V_source to
1.265e-12 relative to the fixed nonzero force scale. The constant1Pa pressure
and smaller mu=.001002 stress scale make cancellation visible; no pressure term
is silently removed or reset.

All native summary geometry/zero-moment/rotated-square gates and disk convergence
criteria are independently reapplied using the analytic source formulas. Finest
disk area error is6.274914e-6; radial/cubic-flux error is1.254978e-5. Pairwise
orders range1.999902–1.999992. These are polygon geometry integration errors,
not solved-velocity/pressure or physical-traction PDE errors. The smallest
retained positive native cell volume is1.27963e-7m³. Native records also expose
727 positive velocity duals with absent open faces; no active-DOF policy follows
from a positive dual volume.

## Exact tangent audit: finite-error pass does not certify topology

Ordinary floating edge intersections initially suggested four extra intervals
where the native face was absent: two at disk32 and two at disk64. Exact rational
intersections of the exported source prove each is a zero-length tangent and
confirm the native absence. These four were independent-oracle arithmetic
artifacts, not native topology errors. They remain recorded under face_ambiguities.

A separate audit then queries every native positive aperture smaller than the
frozen1e-10*h interval budget. This threshold only selects diagnostic queries;
no aperture is changed. Exact Fraction half-plane intersections of the actual
exported f64 operands find five genuinely native positive-aperture artifacts:

| Grid | Axis / native face index | Native positive length(m) | Exact source length |
|---|---|---:|---:|
| 32 | Y / 143 | 9.7076998531e-19 | 0 |
| 64 | X / 2088 | 1.8750264125e-20 | 0 |
| 128 | X / 8272 | 1.7885576809e-20 | 0 |
| 128 | Y / 2111 | 2.6828365213e-20 | 0 |
| 128 | Y / 14400 | 8.9427884043e-21 | 0 |

All retain the declared positive-axis normal orientation. Each exact rational
lower bound equals its upper bound. Maximum length error/h is1.554e-17, so these
artifacts are consistent with the passing finite interval-error budget. There
are five unique geometry artifacts and ten repeated raw records across the two
viscosities; they must not be hidden by a floating agreement statement.

Their native dual volumes are positive: about1.89649e-3m³ at32,4.81208e-4m³ at64,
and1.21186e-4m³ at128. In each case the face touches an empty primal cell and a
nonempty one; all exact bounds, endpoints, signs, dual volumes and adjacent
cell volumes are retained in tangent-audit.json. A future B/pressure graph or
velocity-extension choice could therefore misinterpret the tiny aperture as
connectivity even while every finite flux integral is accurate.

## Admission boundary and handoff

Accept only the stated finite-error geometry/integration evidence. Carry the
five native tangent artifacts as an explicit blocker before exact topology,
active-DOF selection, B/M/S/W assembly or a public production geometry owner.
The next mathematical selection needs a consistent robust source-face predicate
and ownership rule that preserves actual positive cuts; a numerical aperture
floor or epsilon clamp would not discharge this issue. No v3 or native rerun is
part of this review.

Even after that predicate/ownership question, coupled algebra, gauge/component
handling, small-cell policy, solver selection, disk/annulus MMS, true solid
exterior geometry, hydrostatics and geometric interface transport remain pending.
Keep all P1/Q1/v1 findings intact. No physical, GPU or UI claim follows.

Replay `python3 tangent_audit.py RAW_DIR OUTPUT_DIR`, then
`python3 reconstruct.py RAW_DIR OUTPUT_DIR`. The scripts import no candidate
or native evaluator. summary.json retains every raw CSV hash, case/association
counts, numerical maxima, convergence and tangent audit. Oracle moment arrays
for each source are retained as oracle-CASE-nN.npz.
