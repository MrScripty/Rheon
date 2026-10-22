# CV3-A prospective protocol v1 — preparation, not execution admission

All entries here precede selected numerical output. No scenario, phase, finest
level, integration budget or deciding metric may be removed after execution.
The exact source/oracle/resource freeze and independent reviews are recorded in
preparation-freeze.md. Their completion does not expand this handoff to execution.

## Corpus, identities and coverage

| Family | World polygon | Grid origin | n |
| --- | --- | --- | --- |
| rectangle-origin | CV1 rectangle | (0,0) |4,8,16|
| square-17-origin | CV1 square-17 | (0,0) |4,8,16|
| square-17-quarter | same square bytes | (h/4,h/4) |4,8,16|

h=1/n. The computational box is origin+[0,1]^2; all component coordinates and
common half-cell knots use that origin, while the physical source and global
analytic fields stay fixed. Every vertex lies strictly inside both complete
component nodal hulls for every declared case. Derive this directly from the
vertex bounds at n4 (the largest h); shifting the square origin by h/4 retains
both margins. No physical edge coincides with a primal Cartesian plane.

`cases.rs` pins vertex bit patterns to the exact CV1 source.csv files, including
square trigonometric results captured once in the prior immutable campaign.
Rectangle CSV SHA256:
`860b4140b7483797842e6c6bdeb1e661d1e8510fb0855cf9a69b772914208666`.
Square CSV SHA256:
`2a2938a1ae4b12bfcc6c33ff3cb8fc332fc55436e1247c32638e8b83fde1b6cc`.
Export and independently check actual bits, source CSV hash, case/grid/phase,
origin, material, dt, depth and field/basis/load/reconstruction versions.
All source/support/pressure decisions remain exact-positive-area admissions.

SI primitives and continuum derivation are in [mathematics.md](mathematics.md).
Native M/B/K/geometry/support and selected solve come from the existing owners,
with separately reviewed dimension guards only. New body loads use existing
3x3Duffy volume quadrature; new traction loads use three-point Gauss after all
half-cell/P0 splits. Retain original two-point trace maps as existing flux maps,
not as the new traction integrator. Gv is the full vector H1 Riesz integral
of grad(phi):grad(phi)+Lref^-2 phi.phi, with the consistent mass term.

## Raw custody and native observations

One process per exact case, serialized in table order. Each creates a fresh
case directory and exports original source/geometry/bulk operators and maps,
all physical load points, body and boundary vectors separately, actual A,
all selected factor coordinates and iteration0,1,2,3 states with original
residuals flushed after each state. No CV3 unscaled baseline solve is selected.
Retain original full A, including any rounding asymmetry. No floor or fallback.

Native deciding algebraic checks stay at1e-10: dimensionally normalized original
momentum/continuity, physical cell flux, forced work and residual-corrected work,
absolute residual work, direct physical versus nodal body/boundary power and
resultants/torques, rigid virtual-work momentum. Export each signed contribution
so body/boundary cancellation cannot conceal an omitted load. No unforced
energy-decrease gate. Exact defining formulas are the prospective source.
Independent factor identity checks retain CV2's1e-10 budget even though the
MMS exporter does not reuse CV2's fixture-coupled experiment/test adapter.

## Independent solution and assembly comparison

R1 reconstructs exact represented exported A/B/RHS. R2 independently integrates
exact rational represented-source geometry, Q1/P0 M/B/K/H1, continuum body force
and polynomial edge traction; it imports no native geometry/assembly/Gauss code
and uses no exported rounded operators as its own target. It uses the actual
binary64 primitives BEFORE coefficient products/sums. Preserve R1/R2 gap and
the native error against each. Fixed CV2 coefficient budgets remain1e-10 using
Uref=1 and Pref=rho Uref Lref/dt+mu Uref/Lref (distinct from tau_ref=1Pa).

Native-versus-reference comparison alone is not a guaranteed bound. Select the
exact rational inverse-defect/residual certificate in mathematics.md for BOTH
targets. Candidate inverse generation is fixed highprecision LU; precision,
inverse representation, source hashes and commands require oracle freeze.
No failed certificate can be replaced with an estimated budget under v1.

## Physical norms and conservative decisions

For q=1,2, velocity relative error is
`||uh-u||_Lq(Omega) / ||u||_Lq(Omega)` with Euclidean vector magnitude;
pressure relative error is `||ph-p||_Lq(Omega)/||p||_Lq(Omega)` without mean
subtraction. Vector L2 uses the sum of component squares. Traction error is
`(integral_Gamma |sigma_h n-g|^q / length(Gamma))^(1/q) / tau_ref`.
Use liquid-side P0 pressure and the actual component-Q1 derivatives on each
common split piece, never averaged/extrapolated/re-fitted stress. Report normal
and tangential components and per-edge maxima separately. The prescribed g is
nonzero and is explicitly subtracted; no zero-traction claim follows.

Independently integrate all L2 squared polynomial errors by exact moments.
For L1, use bounded polynomial-range/subdivision followed by outward-enclosed
square roots; mere Gauss sample agreement is not acceptance. Freeze its actual
algorithm, depth/work limits and absolute integration budget in the independent
oracle before execution. The selected v1 Bernstein algorithm has depth24, a
case-wide2,000,000-created-piece cap and256-bit outward square roots. Its
absolute L1 width is at most1e-8 times domain measure times the declared unit
velocity/pressure/stress scale, followed by the independent relative1% budget;
this internal measure scaling is not a claim of an exact polynomial L1 rule. Report L-infinity as a lower/upper supremum enclosure
from piecewise polynomial range bounds and point witnesses, with the exact
subdivision definition frozen, not as a claimed cut-face convergence order.

Propagate coefficient, assembly, reference and integration uncertainty to every
error numerator, analytic denominator and traction norm by interval operations.
Each total numerical uncertainty must be at most1% of the positive lower bound
of its reported discretization error AND1% of the refinement difference
interval's distance from zero. A resolved negative coarse difference is reported
with its uncertainty; only the finest-pair order decides. An interval crossing
zero leaves the difference unresolved. A nonpositive analytic denominator
refuses the norm decision; no arbitrary positive floor is inserted.

All six finest n16 family metrics (velocity/pressure/traction, L1/L2) must have
upper bounds<=.05. For each family and each metric report4->8 and8->16 orders;
the deciding pair is8->16. Its conservative lower order is
`log2(E8.lower/E16.upper)`, required>=.9 with positive quantities. Refinement
gap uncertainty includes both levels. Keep exact interval endpoints and the
unrounded conservative decision. Never replace unknown uncertainty with zero.

At each n, also report both phases' norms and the same-domain reconstructed
origin-versus-quarter differences for velocity/pressure/traction. Partition at
the UNION of both grids' knots/P0 boundaries; normalize identically to analytic
field norms/tau_ref. These phase differences have no invented pass cutoff.

## Rank and full H1/L2 diagnostics

Keep every physical pressure row including the constant mode. Require exact
full row rank B, via the frozen exact algorithm. Freeze rank algorithm and
deterministic pivot identity before execution. Gv and Qp=positive exact cut-cell
volumes define `Qp^-1/2 B Gv^-1 B^T Qp^-1/2`; compute the complete spectrum,
minimum-mode pressure pattern and its square-root diagnostic with independently
bounded uncertainty. Fixed alternating pressure is `(-1)^(cell_i+cell_j)`,
Qp-normalized without mean subtraction; report its Rayleigh quotient and any
near-checkerboard mode overlap. Report constant pressure similarly. No beta
cutoff, row deletion, gauge or uniform inf-sup theorem. CV2 A/Schur scaling
spectra remain separate material/dt-dependent diagnostics.

## Always-on evaluator guards and refusals

Before evaluation freeze normal/-O/-OO guards and complete output/report shape.
Negative controls must reach named mathematical guards: omitted/reversed/wrong
pressure traction; wrong/omitted body force; wrong phase or source; wrong P0
trace; two-point degree-four oblique traction; wrong final velocity with
internally consistent rewritten residual metadata. Keep unrelated invariants
valid; parser/nonzero-exit rejection alone is insufficient. Full normal/O/OO
scientific reports must be byte-identical. Retain every case/refusal/attempt.

## Execution and stopping

The [resource plan](resources.md) covers all nine cases before any selected
execution, including rank/highprecision/certificate/L1 cost. A hard cap,
rank/certificate failure or unresolved norm/order interval stops the campaign
with completed outputs retained. No coarsening, posthoc tolerance/precision
change, hidden backend fallback or selected-phase omission. No numerical
command is launched under this preparation-only continuation.

## Prepared commands and interpreter evidence

The native preparation checkpoint is b9ec50c779122587a5da30318b29b42e1825ec10.
Its copied release executable SHA256 is
`72f4245bf56a56da6e8a8c223c2a7a56220fc914491cf6997416c269945f9ad4`;
it has not been executed. `run_native.py` prepares one fixed nine-case sequence
with fresh output, per-case limits, total wall cap and retained stop receipts.
The three new Rust numerical contracts remain explicitly ignored in ordinary
CI; eventual admitted `cargo test ... -- --ignored` commands select them.

The independent analyzer's prospective CLI is `analyze.py case CASE OUTPUT`,
`analyze.py audit CASE OUTPUT`, and `analyze.py aggregate RAW REPORTS OUTPUT`.
`check_oracle.py RAW FRESH_OUTPUT` prepares all nine full case calculations plus
aggregate in each of normal/-O/-OO (30 subprocesses), then14 targeted controls
in each mode (42 subprocesses). Full scientific reports must be byte-identical
even when the scientific method fails a gate. Tooling success cannot relabel a
scientific failure; all cases/modes remain retained. Actual fixed source hashes
and argv are in commands.json and SHA256SUMS-preparation. Check that snapshot
before any later admitted launch; no command here was numerically executed.
