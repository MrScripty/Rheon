# Proposal: ephemeral increments, authoritative stored endpoints

This is a research contract for review, not a solver implementation or an
acceptance change. Its base is frozen `f0b81b4a5f76cb706e645fad4f38faf028c5727e`
(tree `59927a7fb02d949859b78f7e78865d1717f19870`). The five native finer-step
refusals, original temporal-band failures and all initial conditions remain
unchanged. No trajectory is advanced here. The preceding
[arithmetic diagnosis](../../../evidence/forcing-residual-arithmetic/README.md)
only changed inertia in its chart sensitivity probes; those are not complete
equations and cannot authorize a repair.

## Decision proposed for the next research prototype

Use compensated increments **only as ephemeral evaluator workspace**. The
accepted geometry, all three stored nodal velocity components, pressure, clock
and stamp remain the sole persistent state. No compensation tail, latent chart
coordinate, previous acceleration or pressure seed survives publication or
affects the next step. The workspace may compute a candidate more accurately,
but every qualification must bind to the candidate that would actually be
published. Discarded rounding tails cannot supply missing momentum or work.

Call this proposal E. It retains the stored-endpoint interpretation. It does
not promise bit-identical arithmetic or identical accepted trajectories after
a future solver change. Reordering a reduction or changing a chart solve can
move a measured residual across a fixed threshold. The evaluator's arithmetic
policy therefore needs explicit review and versioned evidence even when the
equations, state schema and tolerances are unchanged. Until then, the current
native operation order and all refusals remain authoritative.

In particular, even initially **identical rounded candidate fields** can pass
or fail differently when compensated accumulation changes the measured rate.
Different first-step acceptance then changes every later public trajectory.
The scalar accumulation example below uses identical inputs and the existing
threshold to demonstrate this distinction; it is not a Rheon counterexample.
Ephemeral does not mean behavior-neutral. The independently reviewed diagnosis
reproduced all 110 residual components/five norms bit-for-bit and found the exact
binary squared norms still above `(1e-13)^2`; compensation has not demonstrated
a hidden pass for those native captures. All 220 neighboring observations remain
refused, without establishing a numerical floor.

An alternative L would qualify a latent continuous/compensated state and round
it only for display. L preserves some real equations but changes the state
that the public step advances. If its tails survive, they are retained physical
state, must participate in the existing owner's transaction and memory budget,
and must be exposed in a versioned checkpoint. If they do not survive, each
publication performs a projection/reset that must have its own impulse, energy
and path interpretation. L is a user-facing semantic decision, not an invisible
precision repair. This proposal does not select L or initial-state projection.

## What the current source actually qualifies

References below are to the unchanged source at the frozen base. They describe
runtime code, not a formal proof:

| Source | Current behavior |
| --- | --- |
| `src/coupled_discrete.rs:160,257,464` | Borrows one accepted geometry/velocity/pressure/time/stamp; wraps the existing accepted owner and one candidate workspace. |
| `src/coupled_discrete.rs:1213,1226` | Extracts q and six eta values from geometry and unit embedding rows of stored nodal velocity; extracts 22 planar coefficients z0. |
| `src/coupled_discrete.rs:1340` | Evaluates the quadratic q/linear eta path, reassembles geometry, solves 15 dependent coefficients, embeds velocity and inspects physical operators. |
| `src/coupled_discrete.rs:1623` | Replaces the reconstructed start z, velocity and mass with the accepted values, integrates physical positive/negative mass rates, and evaluates stable and direct endpoint momentum. |
| `src/coupled_discrete.rs:1773` | Gates endpoint energy, BE increment, mixing, strain, pressure, GCL and residual work using unchanged scales. |
| `src/coupled_discrete.rs:840` | Solves/gates the third component on the same endpoint geometry and shared transfers. |
| `src/coupled_discrete.rs:796,831`; `src/translated_viscous.rs:483` | Reassembles/inspects the final candidate and publishes geometry/velocity/time/stamp, then pressure, after the final cancellation barrier. |

The old endpoint is the stored accepted field, not `point(q0,eta0,0)`.
Construction checks reconstruction and all divergence rows within `1e-11`;
it does not project the accepted field to exact chart values. The diagnosis
measured a fresh-start coefficient gap up to `3.45402718772271e-16` for the
initial fixture and zero for the pressure-state fixture. These are observations
on five captures, not global bounds. A path reconstructed from q/eta may therefore
have a small initial mismatch with the accepted momentum field. Proposal E
preserves that existing interpretation and reports the mismatch. Making the
entire path continuously start at the stored field would require an additional
path/constraint decision; imposing delta(0)=0 on an incompatible chart is not
an algebraic repair.

## One explicit evaluator graph

Let B denote binary64 values interpreted exactly as real inputs for analysis.
Let P mean the declared checked binary operation graph, not rounding only once
after an arbitrary real calculation. All intermediate nonzero subnormals,
checked-division failures, overflow and nonfinite values remain refusals.
The following is a specification for a research evaluator, not executable
production pseudocode with an assumed compensated arithmetic library.

1. Borrow accepted `q0B`, `U0B`, `m0B`, `z0B=coefficients(U0B)`, pressure,
   time and stamp. Extract `eta0B` as the current source does. Keep the existing
   embedding R and verify it at each geometry. Record `e0=R z0B-U0B` in an
   exact-input diagnostic. No old-state copy/history or replacement is needed.
2. For each candidate `(alphaB,piB)` and every required s, use the current
   checked path graph

       etaB(s) = P(eta0B + s alphaB),
       qB(s)   = P(q0B + s eta0B[0:3] + (s*s/2) alphaB[0:3]).

   Its operation order is the existing nested source order. Build the cap,
   triangles, masses, pressure basis, strain weights and D from **this qB**.
   Computing a compensated q internally is allowed as a diagnostic, but using
   its low part to assemble a different domain would be a separate arithmetic
   policy; it is excluded from E's first prototype. Existing displacement,
   topology, material, geometry and range gates apply.
3. With the seven known columns K and 15 dependent columns U, form the known
   target `zKB(s)` from the rounded eta values, including the sign of column 13.
   The exact-input increment is `deltaK = zKB(s)-z0KB`, **not** s*alpha whenever
   those differ. Let `A=D_rows,U(qB)` and solve in bounded workspace

       A deltaU = -D_rows(qB) z0B - D_rows,K(qB) deltaK.       (1)

   This is exactly the absolute chart solve for the same known targets and
   same D if the algebra is exact. Retain the accepted constraint defect:

       -D1 z0 = -(D1-D0)z0 - D0 z0.

   Dropping `D0 z0` silently assumes an exactly projected initial state.
   Equation (1) may also be evaluated at s=0; its delta need not vanish because
   the chart reconstruction need not equal the accepted state. Use all 24
   independent measured constraint checks, not only the selected square block.
   The rounded redundant rows need not satisfy exact symbolic dependencies.
4. Form a latent solved coefficient `zhat=z0B+deltaL` and then a **single declared
   candidate** `zB=P_store(zhat)`, retaining the exact known binary targets.
   Reconcile to `deltaB=zB-z0B` in exact-input diagnostics or bounded compensated
   workspace. The discarded `rho=deltaB-deltaL` is observable in momentum:
   `R^T M1 R rho/h`; it must not be omitted or counted as physical acceleration.
   `deltaL` is a solve variable, not the qualified increment. A particular
   finite precision solve, rounding/normalization algorithm, refinement budget
   and byte reservation are intentionally still implementation review items.
5. Embed `UB=P_embed(R zB)`. Inspect geometry using this UB and the same piB.
   Recompute the mesh motion, Ddot, mass rates, relative face mass fluxes,
   convection, full symmetric strain, pressure image/forces and their work
   from that single point. Pressure's adjoint is the same
   `B=-Q^T W_area D` assembled on that geometry. No captured force, flux or
   stiffness from another coefficient vector is reused. Solve/check the
   differentiated constraint using the same D, Ddot and zB; a derivative of
   the real chart is not a derivative theorem for the rounded program.
6. Recompute sign partitions from this candidate path with the unchanged
   bounded brackets/bisections. At every 16/32 quadrature point repeat steps
   2–5. Accumulate both `Fplus` and `Fminus`, the actual path momentum diagnostic,
   all constraints, and the same endpoint pairs. Never use the net transfer as
   a substitute for positive and negative transfers. No new clipping, roots,
   quadrature levels or unbounded adaptive history are authorized.
7. Assemble the **endpoint** donor convection with UB(h), and force/pressure/
   strain on the same stored endpoint. Use the same accepted-mass body load
   `h*m0B*aB`. Evaluate the existing stable residual on reconciled deltaB,
   plus the direct nodal residual on the actual U0B and U1B. The compensated
   workspace can improve arithmetic on these inputs; it cannot replace them
   with latent velocities. Recompute each perturbed candidate through this
   graph for finite-difference Newton columns. Pressure columns remain linear
   only while geometry and velocity are fixed. Retain seven corrections,
   terminal validation, call budget 200 and the current `1e-13` Newton check.
8. Qualify 16/32 refinement, full constraints, GCL, stable/direct `1e-11`
   momentum gates, force impulses and work scales on these same inputs. The
   fine endpoint must be the one reassembled for publication. The third block
   uses that same geometry, m0/m1, Fplus/Fminus, pressure and old third velocity;
   its endpoint, both shears, direct/stable residuals and work stay authoritative.
   E initially leaves the third solver arithmetic unchanged; its existing
   failures cannot be waived by a planar pass. Reinspect the full three-component
   candidate and combine ledgers before the existing final publication barrier.

This defines coherent **stored fields**, while intentionally retaining the
existing native geometry/inspection arithmetic. Exact assembly of continuous
geometry, alternative compensated operators or a new third solve would be
separate prototype policies. Merely improving inertia is insufficient when
the solved zB changes: all downstream terms in steps 5–8 must then be rebuilt.
For the very same zB, an exact-input sum is a diagnostic of those binary fields,
not an independent continuum/geometry oracle.

## The two momentum residuals are not identical on rounded nodal fields

For exact algebra on the declared binary inputs, suppressing planar component
indices, write

    I_stable_i = m1_i R_i(z1-z0) + (m1_i-m0_i) U0_i,
    I_direct_i = m1_i U1_i - m0_i U0_i,
    e0_i = R_i z0-U0_i,       e1_i = R_i z1-U1_i.

Then **even before reduction roundoff**,

    I_stable_i-I_direct_i = m1_i(e1_i-e0_i).                  (2)

Thus the familiar rearrangement is equivalent if the two embedding defects
are equal (in particular both zero). Unit rows are exact, but interpolated
stored nodes can carry rounding discrepancies. The direct/stable comparison
cannot be described as summation error alone. Keep both current gates and
report (2), rather than force the nodal field onto Rz or silently change which
residual Newton tests. Norms have units of reduced momentum rate; a coefficient
gap divided by h has different units and is not a Newton threshold test.

## Ledger binding and publication rounding

Use actual stored full vectors U0/U1 and geometry-derived m0/m1 for

    Tn = sum_i mn_i |Un_i|²/2,
    D_BE = sum_i m0_i |U1_i-U0_i|²/2,
    D_mix = sum_faces (Fplus+Fminus)|U1_i-U1_j|²/2,
    G_i = m1_i-m0_i+sum_j(Fplus_ij-Fminus_ij),
    W_G = sum_i G_i |U1_i|²/2,
    W_ext = h sum_i m0_i a_i dot U1_i.

Strain and pressure work use the same endpoint operators and full stored U1.
The existing report definitions and `128*eps` scale are retained, including
both shears, all absolute work terms, pressure-work and nonnegative-loss gates.
The planar reported residual work is currently `h*z1 dot rate_stable`; the
third and combined reports retain their source definitions. Do not replace
that pairing with latent endpoint velocity or drop a rounding discrepancy.

For an exact nodal construction, let
`rN=I_direct+c_BE+h*f_strain+h*f_pressure-h*m0*a` be the integrated residual.
Pair antisymmetric transfers with U1 to derive locally

    T1-T0+D_BE+D_mix+W_mu+W_p+W_G-W_ext = U1 dot rN.          (3)

This is the changing-mass identity, not a temporal/continuum theorem. If the
reduced stable residual is `R^T[rN+m1(e1-e0)]`, its coefficient work is
`W_R=(U1+e1) dot [rN+m1(e1-e0)]`. The exact-input ledger discrepancy relative
to that pairing is therefore

    U1 dot rN-W_R = -e1 dot rN
                   -(U1+e1) dot m1(e1-e0).                 (4)

Measured binary strain/pressure adjoint discrepancies and arithmetic reduction
errors add to (4); the reported powers need not equal an independently exact
nodal force pairing. These are measured diagnostic discrepancies, not extra
physical losses or new allowances. Equation (4) explains why the current ledger
gate remains necessary even after a tiny stable residual. It cannot authorize
a residual bypass. The exact small examples exercise (2)–(4) and a reversed
transfer pair; they do not claim these defects explain the five native refusals.

If latent state were selected instead, rounding would have to be accounted for
on both momentum and energy: for fixed mass,
`DeltaP_store=sum m(U_B-U_L)` and
`DeltaT_store=sum m(|U_B|²-|U_L|²)/2`. Geometry rounding would also change masses,
operators and path transfers, so those formulas alone are insufficient for a
moving domain. Relabeling these as viscosity or BE dissipation is incorrect.
Under E, stored endpoints enter the equation/ledger from the start; these are
diagnostic comparisons, not corrections added to manufacture acceptance.

## Observable state, replay, restart and transaction

On successful future execution, the existing borrowed state remains the public
authority. Volume remains fitted nodal mass divided by fixed density; no phase
owner, latent liquid state or second clock is added. Reports must refer to the
stored end_q, end_eta, actual velocity, pressure, time/stamp and their inspected
geometry. Compensation tails can be diagnostic, but must be labeled discarded
workspace and must not be used by replay, later seeds or subsequent steps.

There is **no implemented coupled checkpoint/restart API** in this source.
PNG guidance export and the temporal probe's JSON replay are not restart
formats. Calling a constructor is not a bitwise restart: it checks inputs and
seeds pressure instead of restoring a saved pressure/time/version transaction.
For a future restart contract under E, a versioned record must preserve exact
bit patterns of the authoring geometry/material parameters (or an equivalent
complete frame description), all nodal velocity components, pressure, time,
stamp, solver/resource settings and evaluator-policy identity. Reassembled
topology, operators and masses must reproduce the declared stored frame or
reject an unsupported/incompatible checkpoint. Signed zero cannot be silently
canonicalized unless explicitly versioned. Bit parity additionally depends on
the declared arithmetic/toolchain policy; none is promised by a nonexistent
serializer. No previous alpha/delta/tail is needed under E.

The same next call from equal stored inputs/settings/loads must produce equal
observable behavior whether ephemeral workspace was newly allocated, dirtied
by cancellation or reused. Failures must leave all accepted bits and stamps
unchanged. A later prototype must initialize every workspace value it reads,
borrow old state, construct one candidate, and publish via the existing final
barrier. Any fixed compensation lanes, solve factors or diagnostic buffers
must be added to the actual simultaneously live reservation and nominal bytes
with checked sizing. Current STEP_STACK_BYTES is not proof that added storage
fits. No heap history, hidden accepted snapshots or parallel state owner is
authorized. Arithmetic tail underflow must still reject nonzero subnormal
intermediates; compensation is not permission to bypass current range guards.

For L, a restart omitting the low parts can change future **public** endpoints
even when its saved high parts match. Hidden low state is therefore forbidden:
it must be explicit, budgeted, transactional and serialized if selected. L
would need a versioned explanation of latent vs displayed energy, pressure,
geometry, liquid volume and rounding exchange. This is the principal semantic
choice requiring user-facing review; E avoids it. Initial projection and a new
path starting exactly at an off-chart accepted state are separate choices too.

## Exact examples and discriminating tests

[Executable examples and results](../../../evidence/forcing-increment-contract/README.md)
use Python Fraction as a small exact oracle and native Python binary64 rounding.
They perform no Rheon solve/advance. The first example has U0=1, m=1, a=1 and
h=2^-54. Binary storage gives U1=1 although latent U1=1+h. The latent equation
has zero residual; the endpoint momentum rate is -1. Pairing endpoint energy
with latent residual falsely loses work h. This separates real equivalence from
qualifying a different endpoint. Three retained quarter-ulp increments produce
a different displayed endpoint from three discarded increments; a checkpoint
omitting the retained tail changes the next displayed result. Another two-column
analytic constraint solve checks the initial defect term in (1). These small
values are normal and exact; they neither bound the native model nor prove a
floor at any interval.

Before a production implementation is considered, the following tests should
discriminate E from an inconsistent hybrid or L:

| Test | Required observation / corruption to reject |
| --- | --- |
| Lost increment | Qualify stored deltaB, not h*alpha or deltaL; endpoint rate -1 in the scalar example despite latent zero. |
| Moving constraint | Exact absolute/increment solutions agree for the same q/D/known targets; dropping accepted `D0*z0` produces a nonzero final constraint. |
| Rounded geometry | Assemble masses/D/B/strain/flux from qB throughout; a latent q used for only one term is rejected. |
| Embedding defect | Stable/direct gap matches (2) on exact binary inputs; neither gate is hidden or renamed roundoff. |
| Work | Independently derive (3), measure (4)/adjoint discrepancy; mixed latent residual and stored energy fail. Preserve both directional transfers. |
| Complete recomputation | Change any solved endpoint coefficient; recompute pressure image, mesh motion, strain and transfers. Stale force/flux/third geometry must be detected by independent full equation/ledger checks. |
| Precision cross-check | Small exact rational cases; complete fixed-input high-precision diagnostics at two precisions. Captured-inertia-only improvement is never an acceptance candidate. |
| Lifecycle | Dirty/new workspace and cancellation/failure/retry on nonzero accepted state give identical accepted bits and continuation; final barrier atomically binds velocity/pressure/geometry/clock. |
| Restart | For a future real checkpoint API, bitwise stored state plus evaluator/settings reproduces continuation; omitted required field/unknown policy rejects. Constructor/replay are never labeled checkpoint parity. |
| Arithmetic/resources | Checked divisions, nonzero subnormal tail/product, finite range and all live byte/call/iteration bounds reject correctly without altering accepted state. |
| Public counterexample | Reuse one unchanged original refusal input in an isolated research prototype; record actual public result, both residuals, all physical gates and stored output. Do not claim a repair before this exists. |

Only the exact small examples are executed by this packet. The native prototype,
bounded compensated arithmetic, complete high-precision geometry/force evaluator,
public-call counterexample, lifecycle/memory validation and restart API remain
unimplemented. No acceptance promotion, temporal rerun, numerical floor, new Lean
theorem or general 3D/material/surface/anatomy claim follows.

## Sources and proof boundary

The quadratic path, finite endpoint donor equation and changing-mass identity
are the existing [finite-work experiment](fitted-discrete-work-temporal.md).
Equations (1)–(4) above are local exact-algebra derivations under their stated
inputs; the executable examples check them, but they are not Lean proofs.
The existing [finite-arithmetic chapter](../chapters/15-floating-point.md) and
its S18 [authorized Goldberg reprint](https://docs.oracle.com/cd/E19957-01/806-3568/ncg_goldberg.html)
distinguish rounded operations, representation error and exceptional range.
That source motivates careful arithmetic policy; it supplies no theorem about
this solver or arbitrary compensated workspaces. Existing citations and exact-
real Lean claims remain unchanged, including every moving-liquid limitation.
