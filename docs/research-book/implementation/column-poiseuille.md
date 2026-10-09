# Uniform tangential forcing and Poiseuille startup

`ColumnShearWorkspace::update_with_forcing` adds one uniform tangential vector
to the compatible fixed-wall column operation. `UniformColumnForce` uses the
existing `ForceUnits` enum: acceleration in m/s² or force density in N/m³.
For actual liquid dual volume V_j=A omega_j and m_j=rho V_j,

    acceleration: f_body,j=m_j a.
    force density: f_body,j=V_j q, with equivalent a=q/rho.

The implementation assembles these products directly using the selected units.
It does not apply the same numeric vector as both quantities. Positive constant
rho, finite normal-or-zero vector components and zero normal force are required.
No support regions, pressure gradient solve or variable-density inertia is added.

## Actual simultaneous explicit step

On the unchanged endpoint basis, an edge transfers k(u_{j+1}-u_j),
k=mu A/h. Its two equal/opposite node contributions retain internal momentum.
Finite Navier traction is assembled next, then body force on every liquid dual
node, including constrained endpoints. A free node has

    m_j delta_j=dt(f_bulk,j+f_Navier,j+f_body,j).

At a compatible fixed no-slip endpoint delta_j=0 and

    J_j=-dt(f_bulk,j+f_Navier,j+f_body,j).

This is one combined explicit step, not a force/viscosity splitting scheme.
No hidden substeps or retries are supplied. Free rows retain the existing
dt(sum adjacent stiffness+incident beta A)/m <= 1 admission; a constant
source does not change the homogeneous row coefficients. The unforced convex
velocity interval is enlarged by the signed dt f_body/m for each row, with
the corresponding rounded-f32 interval checked separately. There is no clamp.
Normal/dry velocities stay zero. All checks/callbacks precede output publication;
the same five fixed scratch vectors are reused with no per-call allocation.
Old APIs and zero-forcing results remain compatible.

`ColumnForcedShearReport` contains the original boundary report and supplied
force units/vector, total body force, body impulse and signed old-speed work.
Its nested `shear.force_sum` is the net force after constraint elimination,
including body forces and wall reaction. Body totals include the liquid dual
volume at constrained nodes; omitting those nodes would misstate wall reaction.

## Momentum and work

The stored-velocity ledgers retain all terms independently:

    Delta momentum = body impulse + total wall impulse + rounding momentum.
    W_body,old = dt sum(f_body,j dot u_j).
    Delta T + dt(P_bulk+P_Navier) - W_wall - W_body,old
      - sum(m_j |delta_j|²)/2 - rounding_work = 0.

Work delivered by moving walls and body old-speed work can each be negative.
Stationary walls transfer momentum but perform zero work. From rest, the first
body old-speed work is zero while explicit increment energy is positive. The
squared-increment correction belongs to the combined step, including cross
terms; it cannot be assigned solely to a separately applied body-force stage.
This convention differs from midpoint applied work in the existing box force
stage. Runtime budgets retain the original 64-epsilon identity policy and
include the actual body-force and absolute body-power scales.

## Steady oracle and finite-time behavior

With stationary no-slip walls at 0,H, uniform q=G, constant positive rho/mu,
zero normal motion and periodic lateral directions, the continuum equation is
rho u_t=mu u_yy+G. Its steady profile and flow per unit width are

    u(y)=G y(H-y)/(2mu),       Q'=G H³/(12mu).

No pressure field is solved here. A pressure-gradient drive G=-dp/dx has the
same tangential momentum source under these assumptions. Holding G fixed gives
the same steady speed at different rho, but relaxation changes with nu=mu/rho.
Holding acceleration a fixed makes G=rho a, changing steady amplitude as well.
Doubling mu at fixed G halves steady speed and changes the relaxation rate.

The retained basis fixes wet-center endpoint values and extends them constantly
to the physical wall. For full dual lengths h=H/L its distinct equilibrium is

    u*_j=G h² j(L-1-j)/(2mu),      j=0,...,L-1.

The physical continuum reference is evaluated at y_j=(j+1/2)h; these are
different references. Decreasing endpoint deviation is spatial evidence, not
temporal-order qualification. Partial dual lengths are covered by contracts,
not by this full-layer analytic equilibrium.

From rest, the exact-real explicit recurrence has an independent closed form.
For M=L-1, b_k=(2/M) sum_{j=1}^{M-1} u*_j sin(k pi j/M),

    lambda_k=1-4 nu dt/h² sin²(k pi/(2M)).
    u_j^n=u*_j-sum_{k=1}^{M-1} b_k lambda_k^n sin(k pi j/M).

`column_poiseuille` evaluates these finite sums at saved times; it never
integrates a second reference trajectory. This exact-real discrete startup
reference compares native storage/arithmetic against a binary64 evaluation of
the closed-form exact-real discrete recurrence. Observed deviation includes
f32 storage, native assembly rounding and reference sin/power/sum rounding;
no certified reference-evaluation bound is supplied. It does not measure
error relative to a continuum transient or establish time order. The
runtime row condition bounds the homogeneous amplification; no new formal
spectral convergence or IEEE theorem is claimed.

## Recorded native laboratory

Build `cargo build --release --no-default-features --example column_poiseuille`
and run the executable with a fresh directory outside Git. It writes JSON,
seven CSVs and a standalone HTML playback lab. Case/time controls select actual
saved native profiles and ledger entries, with a visible snapshot-playback
notice. They do not recompute parameters, interpolate a trajectory or provide
a general solver. Native, continuum steady, discrete steady and exact-real
discrete startup references remain visibly distinct.

All cases use H=A=1, initial velocity zero, stationary walls, 8192 steps through
96 s, dt=96/8192 s, and save steps 0,128,683,2731,8192.

| Case | Layers | rho (kg/m³) | mu (Pa s) | Supplied quantity |
| --- | ---: | ---: | ---: | --- |
| q-8 | 8 | 3 | 0.15 | q=0.3 N/m³ |
| q-16 | 16 | 3 | 0.15 | q=0.3 N/m³ |
| a-16 | 16 | 3 | 0.15 | a=0.1 m/s² |
| q-rho6-16 | 16 | 6 | 0.15 | q=0.3 N/m³ |
| a-rho6-16 | 16 | 6 | 0.15 | a=0.1 m/s² |
| q-mu2-16 | 16 | 3 | 0.3 | q=0.3 N/m³ |
| q-negative-16 | 16 | 3 | 0.15 | q=-0.3 N/m³ |

Predeclared gates: saved discrete-startup deviation and final equilibrium
deviation <3e-5 m/s; saved cumulative energy/momentum residuals <1e-9 SI units;
baseline continuum deviation decreases 8→16 and every 16-layer deviation is
<0.031 |G/mu|/2 m/s. Final equivalent-unit and sign-reversal profiles agree
within 2e-7 m/s; fixed-G density, fixed-a doubling and doubled-mu controls agree
within 3e-5 m/s with their respective steady predictions. At step683, fixed-G
density doubling has lower peak speed and fixed-a doubling has higher peak
speed than the baseline. These new engineering gates leave earlier gates intact.

The frozen original run passed all gates. Baseline continuum deviations at
8/16 layers were 0.05859480798 / 0.03027459979 m/s, and their separate discrete
equilibrium deviations were 1.057982445e-6 / 1.162290573e-6 m/s. Across all seven
cases the largest saved discrete-startup deviation was 4.738568319e-6 m/s.
At step683 (8.00390625 s), the 16-layer baseline peak was 0.2162252367 m/s;
fixed-G density doubling gave 0.1947998554 m/s, whereas fixed-a density doubling
gave 0.3895997107 m/s. The two baseline unit formulations produced identical
stored profiles. Generated receipts and full data remain outside Git.

## Proof assumptions and scope

`Rheon.ColumnForcing` proves exact-real unit conversion, compatible reaction
including body force, finite momentum cancellation, forced work, zero parabola
endpoints, quadratic-stencil balance and recurrence about equilibrium. Mass/
volume meaning, the step equation, internal-force cancellation and assembled
power are explicit inputs. Polynomial identities do not prove derivative
assembly, Rust floating point, the DST reference or transient convergence.

This is a bounded fixed-column force/viscosity/wall coupling. It does not add
arbitrary collision-mesh fluid geometry, pressure composition, evolving liquid
interfaces, adhesion, wetting or contact lines, and does not complete the
capstone integration roadmap. Historical numerical refusals and outputs remain
separate and preserved.
