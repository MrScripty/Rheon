# Q1 frozen pre-evaluation protocol v1

Canonical plan PR120/8422f77 grid Q1 targets remain unchanged. User adopts Q1→Q2
and stops P1 selection; route-handoff.md is the parent-owned shared lifecycle
handoff. Exact exclusive writes in admission.md are confirmed: one existing
numerical owner src/liquid_volume.rs, two new focused tests, new grid_verification
example/modules, q1 docs. No G1/shared lifecycle/exports/dependencies/proofs edits.

## Native methods and source identity

Existing StaticObstacleShear/StaticObstacleGeometry, backward Euler and stationary
no-slip channel are reused unchanged. Existing shared-face first-order donor-cell
LiquidVolumeState is reused with explicit `advance_periodic` boundary version1;
no higher-order reconstruction, clipping, mass reset or pressure/momentum coupling
is added. Report.periodic distinguishes the new topology; report.inlet retains
old box-inlet identity when false and identifies periodic boundary policy when
true. Existing prepare_advance callers and old arithmetic ordering stay intact.

Paired normal velocities must match as f32 values (signed zeros may compare equal).
Compute lower periodic seam transfer ONCE, copy it bitwise to upper; wrap donor
indices, keep one-cell axes valid. Periodic external influx/outflux are zero;
signed source ledger remains unchanged. Reject slab/reconstructed domains, stale
start, mismatched velocities, excessive CFL/divergence/bounds/ledger or cancellation
before publishing any accepted fraction/time/stamp. No native panic or clamp is
a normal refusal mechanism. Memory Vec accounting remains the existing owner.

Commit final source/protocol/evaluator and independent pre-run review before
ANY selected tests or scientific execution. Earlier d6bf4e1 is a prequalification
source checkpoint, not the numerical freeze. Native Linux x86_64 release f64
fractions/channel and f32 face carriers are explicit. Record actual source/binary,
commands, toolchain, geometry, dt, refusal/last-accepted state and raw fields.

## Channel: preserve original failure and separate time error

Exact frozen-audit replay: rho998.2,mu.001002,nu=mu/rho,gapH=.001m,lowerwall
.001m, tangential extents.001m,bodya=.01m/s². Mesh lower box uses the original
triangle winding/stamp321 and full cross section. n=8/16/32 wet levels; initialize
u=a*y(H-y)/(2nu) at true fluid centroids,20 BE steps dt1s toT20s. RMS is relative
to the same pointwise analytic profile as audit119. Retain n8's2.139313%>2% fixed
failure. Resolved n16/n32 target<=2%; observed spatial orders1.8–2.2 for aligned
smooth case. Do not relabel all resolutions accurate because fine passes.

Original temporal replay: n32 sine mode amplitude1m/s, no force,T=.05s,8/16/32
steps. Exact semidiscrete eigenvalue4sin²(pi/(2n))/dy²; compare toexp(-nu lambda T),
not a continuum/spatial-floor mixture. Original ratios<.65 and new first-order
observed temporal order>=.9 remain binding. Energy cannot increase. This exact
unidirectional operator fixture is numerical verification, not a turbulent-water
validation. Report both RMS and relative RMS and actual fields.

Report physical channel flux Q=.001 sum_j dy*u_j versusa*.001*H³/(12nu), each
wall traction from native boundary conductance andu/area versusrho*a*H/2, and
native residual/momentum/energy ledgers. Retain every native report field, every
accepted and initial layer profile, actual native volumes/masses/conductances in
channel-ledger.csv, channel-states.csv and channel-geometry.csv; maxima alone are
not sufficient. Q1 does not infer general3D stress from
this reduced channel. Native algebraic targets: force residual/(rho*V*a)<=1e-10,
energy identity/E_ref<=1e-10 and momentum identity/P_ref<=1e-10, withV=1e-9m³,
U_ref=aH²/(8nu),E_ref=.5rho V U_ref²,P_ref=rho V U_ref, all fixed before execution.
Also retain the unchanged existing liquid_volume example's4 slab/CFL diffusion
comparators as auxiliary evidence, distinct from audit119's prior3D export.

## Interface: actual native periodic comparator

Grid[0,1]^3, n=16/32/64, R=.15m, source water density998.2 (represented mass only;
carrier inertia/pressure is NOT coupled). Nominal summed-outward C=.1/.25/.5
independently at each resolution. Actual source f32 fields determine maximum
percell outward rate S=max_i sum_d(max(u_hi,0)-min(u_lo,0))/h. dt_nom=C/S;
shorten only to exact phase/end times. Native safety max_outward_courant=1 and
actual_divergence_limit=0; record actual C of every interval. Scientific nominal
C matching permits16*f64EPS relative representation error, not a physical CFL
relaxation. A violation/refusal is retained; no velocity/fraction/source fixes.

Translation: initial spherecenter(.65,.5,.5), u=(.25,.125,.0625)m/s,T1s. Final
center(.9,.625,.5625), with periodic images; the final sphere crosses the X seam.
Vortex adaptation: initialsphere(.35,.35,.5),T1s. Vertex streamfunction
psi=sin²(pi x)sin²(pi y)/pi, boundaries exactly0, quantized to integers*2^-20.
Build u=Delta_y psi/h,v=-Delta_x psi/h,w=0 as exact dyadic f32; assert integer
representability and EXACT discrete divergence0 before native use. It defines a
2D RT0 curl field extruded through the3D sphere, held static for half a period
then reversed with the same dt sequence backwards, shortened atT1. If accumulated
time roundoff leaves a positive remainder after that sequence, one tiny end
interval is appended; it never enlarges the requested maximum dt. Actual dt/time
are retained, including forward/reverse sums derived from each ledger; their few-ulp
finite-time mismatch is reported rather than called exact operational reversal. This accounts for finite-time representation without a fraction or
velocity correction. Exact autonomous flow reversal returns the initial
set. Quantization bound at vertices q/2 and velocity q/h is separate from
transport error. This is an explicit bounded adaptation of Enright/Fedkiw/
Ferziger/Mitchell, NOT the full3D/T3 or long-stretch single-vortex benchmark.
Retain actual f32 carrier arrays, forward/reverse rates and dyadic div checks.

The18 requested trajectories must reachT1 to pass their accuracy gate. On refusal
retain exact error/cell/value/start/attempteddt and final accepted f64 field, set
requested-final shape/Fourier errors UNAVAILABLE, and do not count early time as
completion. Before vortex mixed runs,18 one-step Full/Dry controls (3n*3C*2fills)
check the raw arithmetic at requested dt: require completed interval and max
constant defect<=1e-12. Refusal is a failed control, never clipped away. Unit
contracts additionally check a small dyadic-step invariant and signed seam
impulses on>=3 cells. No live accepted state or benchmark is rerun after seeing
failure without a separately versioned, reviewed implementation rationale.

Fixed interface gates: source-free relative represented-volume drift<=1e-10
against ACTUAL initialized V0; f in[0,1] within1e-12; finest completed relative
L1 error E=sum_cells Vcell*|f-f_exact_cellavg|/Vsphere<=.05; finest pairwise
observed spatial order>=.9 (report both pairs separately). Full/Dry controls,
requested-time completion and source oracle must pass for a positive scope.
Record periodic Fourier moments; ordinary centroid across a seam is not used.
No universal monotonic-dt claim; preserve the existing donor diffusion evidence.

## Geometry and independent uncertainty gate

Sphere initialization/final reference use actual cell averages including periodic
images, no center mask or global mass renormalization. Rust reference integrates
exact disk/rectangle cross-sectional area inX. Quarterprimitive I(t)=[t sqrt(r²-t²)
+r²asin(t/r)]/2 and split at allX edge/corner contacts, center and sphere endpoints.
FULL/EMPTY branches follow independent nearest/farthest distance semantics.
Adaptive Simpson depth22; eachcell absolute tolerance1e-12*h², paired tolerance/16.
Depth exhaustion is explicit and prevents positive admission. Store BOTH arrays,
empirical quadrature estimates and known analytic Vsphere=4piR³/3. These estimates
are NOT rigorous enclosures. No volume/fraction clamp is part of initialization;
source outside[0,1], exhaustion or gross geometry failure refuses the fixture.

Before any positive scientific disposition require, for EACH of9 geometry fields:
- reported empirical sum estimate plus paired normalized L1<=1e-4;
- analytic total sphere-volume defect/Vsphere<=1e-4;
- independent percell source integral oracle: sum_cells Vcell*|fRust-fOracle|/
  Vsphere plus oracle's aggregate integration estimate<=1e-4. A separate positive
  quadrature/vertical-slice implementation must check partial cells and independent
  FULL/EMPTY predicates, not just a total-volume cancellation check.
Native drift versus actualV0 is independent of geometric mass uncertainty.
Uncertainty in shape includes initial+target geometry budgets. Require finest
E+uncertainty<=.05 and lower pairwise order from error intervals>=.9; if those
intervals overlap target/order thresholds the result is unresolved, never pass.
For refusals/uncompleted paths no target-time continuum error/order is asserted.

## Qualification and dependency disposition

A1 focused native policy/rollback/geometry/carrier contracts; A2 actual CPU native
channel and transport data, retained failures; A3 independent discretization,
source geometry and field/ledger reconstruction. Source-owned tests do not replace
A2/A3. Relevant old volume/step/slab/column regressions and full no-default release
CPU regression are required after the production-owner extension; real-checkpoint
ignored assets remain explicit, no surrogate substitution.

Q1 may close with explicit failed accuracy/support rather than false admission.
Actual evidence then admits the smallest Q2 common-geometry/flux/stress reference,
with exact shared zeroth/first/boundary moments and B/M/S/W coupling. Keep pressure
absolute on free-traction components and apply gauges only to appropriate sealed
components. Do not widen to3D mesh water, geometric transport or coupled Stokes
accuracy until the corresponding fixed convergence/flux/physical-stress gates
pass. No third material family, UI/Session/recording, GPU or physical validation.
