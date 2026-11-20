> Historical proposal, superseded by agreed owner-contract.md and plan.md.

# Proposed owner contract — awaiting PR55 response

This proposal is for parent/PR55-owner reconciliation. No item below is presented
as an owner agreement. PR55 was inspected at3525c979, still draft, targeting
feat/material-simulation-api rather than main. Confirm a frozen implementation
base and shared-file write ownership before changing production code.

## Numerical boundary

A seeded solve still validates input/settings/cancellation, full closed RHS
compatibility and lengths in the reference order. Preserve the RHS-based stopping
threshold max(absolute, relative * ||rhs||), not a seed-residual-based tolerance.
Initialize pressure from an eligible accepted seed, pinned to the same closed-box
gauge, then compute r = rhs - A*p with the actual current operator. Subsequent
PCG, true residual, predicted divergence and corrected f32 acceptance gates remain
unchanged. A smaller iteration count alone is not an accepted performance claim.

The cold path remains explicit/default. No seed normalization, dt rescaling,
mean-shifting, altered sums or nonzero air storage is smuggled into this slice.
Closed accepted pressure already has gauge cell +0. Seeded results may differ in
last bits from cold convergence: this is an explicit numerical variant, not a
claim of cold bitwise equivalence. Each mode must be deterministic under its
stated execution contract and satisfy independent equal-accuracy checks.

## State and eligibility

Proposed cache is derived state at the accepted simulation owner, containing
pressure values, accepted generation and an operator/domain identity. It is not
PressureWorkspace::pressure(), a successful PressureReport, or a provider report's
locally qualified pressure. The owner identifies which pressure actually belongs
to the committed corrected fields.

For first slice, eligibility requires native CPU sealed-box pressure with unchanged
geometry (counts, spacing, origin), density, operator/domain mode and gauge policy,
and provenance matching the currently accepted state. Bind preconditioner/method
policy as required by the owner's implementation identity. Do not infer identity
from vector length, allocation address or an unrelated session revision.

| Event | Attempt behavior | Accepted-cache effect |
|---|---|---|
| First eligible step | Cold solve | Publish qualified native pressure only with successful field commit |
| Same-identity subsequent step | Borrow last accepted seed; mutate only scratch | Replace seed only with successful whole-step commit |
| Pressure or later gate fails/cancels | Return existing typed failure; no publication | Prior accepted seed and validity unchanged |
| Attempt uses another domain/route | Bypass seed; ordinary cold/reference path | Failed attempt preserves old seed; successful incompatible commit invalidates it |
| Provider/boundary/column step commits | Cold for this proposed warm mode | Invalidate native seed; never copy stale carrier scratch |
| Reset succeeds | Accepted fields reset by owner | Clear validity atomically with reset |
| Pause/resume without state change | Preserve existing pause behavior | No implicit seed replacement |
| Construction/budget failure | No partially published owner | No retained cache visible |

A changed attempt does not itself rewrite accepted provenance. This distinction
is needed so a failed different-route attempt followed by a retry of the prior
accepted domain does not destroy a valid cache. The owner must define any actual
configuration mutations that commit outside stepping and invalidate immediately.

## Ownership, API and memory decisions to reconcile

1. **Base/write set:** freeze PR55 head or an agreed descendant/stack. Which exact
   Simulation commit/reset and route-provenance fields may this CPU agent change?
   Do not import or replace AI/provider work as part of numerical optimization.
2. **Accepted pressure source:** can the owner expose/reuse a stable accepted native
   pressure snapshot, or must this slice retain a derived array at the owner?
   LiquidTransport already has accepted pressure, but its lifetime must not be
   guessed or duplicated into an authoritative second copy. Standalone Simulation
   currently exposes only solver scratch for pressure.
3. **Opt-in and budget:** propose an additive constructor/options path that leaves
   existing cold calls untouched. Charge any retained Vec capacity (at least
   8 bytes/cell for a new f64 seed array) to existing caps/reports before allocation.
   Agree reset and disable semantics; no hidden allocation on ordinary steps.
4. **Failure budget:** propose one seeded PCG attempt under the existing max-iteration
   cap, with unchanged typed failure and rollback. No automatic second cold solve
   or doubled iteration budget. Cold reference remains separately callable. If
   production contract requires fallback, define its budget/reporting explicitly
   before implementation; benchmark every attempt and failure rate.
5. **Coupled publication:** a carrier solve or prepare is not a liquid commit.
   Confirm the seed copy/invalidation hook runs only after all outer-owner gates
   and uses the pressure source selected by that committed outcome. If first-slice
   opt-in is standalone only, guarantee nested carriers remain cold.

## Concrete responses that would block or change implementation

- No infallible owner commit hook can publish/invalidate seed with accepted fields.
- Accepted pressure provenance cannot distinguish native/provider/boundary routes.
- Cache memory cannot fit the existing declared caps without changing supported
  configuration semantics, and no safely owned accepted buffer can be reused.
- The proposed opt-in mode conflicts with provider selection or iteration-budget
  guarantees that the owner cannot reconcile in a bounded slice.

These are questions to settle with source/owner evidence, not assumed blockers.
The parent has already requested the owner's contract; this proposal should be
reconciled with that response rather than treated as a duplicate permission flow.
