# Accepted-state CPU pressure warm starts

Status: Active — 2026-10-21 current-main integration. Overall acceptance remains open.
Canonical operation: continue this plan under the fresh user request to complete PR59.
The original evaluation-only/no-merge instructions below are historical; the new
request authorizes normal reviewed serial integration after #142 → #138 → #143.

Current binding decisions and prospective protocol:
[integration-20261021/protocol.md](integration-20261021/protocol.md).
Explicit Cargo feature `accepted-pressure-warm-start` contains the optimization;
ordinary builds exclude warm implementation. This transparently narrows the build
scope in response to the retained adverse default-particle evidence. No universal
speedup or production acceptance follows from source integration.

Owner: PR59 worker; main integration owner: PR142 lead. Preserve shared branches.
Initial original head7f1733f and main15026a9 are retained; current-main additive
force/provider conflicts are resolved in integration/pr59-20261021. Readiness is
reported on PR59; merging requires the lead's verified exact-head authorization or
the lead performs it. Dependency order and final main source must be rechecked.

Current phase: waiting for ordered integration after preliminary verification.
Exactly one next slice: compose verified #142→#138→#143 main and qualify that final
source. Preliminary source review and default/warm regressions passed. Acceptance: partial; final default controls, source admission,
exact-source review, required CI and post-merge checks remain pending.

## Agreed scope and admission

The parent relayed PR55 owner's agreement, recorded in owner-contract.md.
Independent review found no design conflict. This agreement supersedes the
historical contract-proposal.md. First slice is LiquidTransportSimulation and
SimulationSession, reusing the existing accepted pressure array. Standalone
Simulation remains cold; no extra seed array is allocated. Both existing native
preconditioners retain their algebra; performance admission is measured separately.

Write set: src/pressure.rs, src/pressure_initialization.rs, src/liquid_step.rs,
src/simulation.rs (internal preparation seam only), src/session.rs, src/lib.rs,
focused tests, example measurement harness and this plan directory. Provider
selection/fallback code, particle/boundary implementations and book stay untouched.

The lower solver borrows an explicit initial guess and certifies no provenance.
Only the coupled owner qualifies accepted history and publishes its metadata,
after all downstream gates and the final cancellation callback. Existing APIs
remain cold; successful native cold steps may establish future seed eligibility.
Unsupported successful publications invalidate; failed attempts preserve history.

Identity uses a checked, constructor-only process-local incarnation and exact
immutable geometry/density/method/gauge key, plus accepted generation and operator
revision. It is not portable replay identity. No registry or retained pressure
copy is introduced. Public accepted reports distinguish initialization from
provider selection. Session capability version changes; dense3D does not encode
warm history. No automatic retry is introduced.

## Objective and authoritative constraints

Reduce sustained equal-accuracy whole-step CPU cost using the previous accepted
pressure as an explicit opt-in initial guess. The cold reference remains the
default and preserves its deterministic arithmetic, tolerances and public calls.
No summation-order experiment, fused-pass tuning, AI/provider replacement, physical
horizon reduction, merge, deployment, book changes or silent retry-budget increase.

Existing evidence remains immutable on PR58 at5038087 and assessment branch
perf/cpu-cache-assessment at799662e. The rejected experiments do not establish a
warm-start speedup. Existing iterative-cost profiles justify investigating a seed,
not retaining it without fresh equal-accuracy evidence.

## Milestones

1. Contract: draft proposal, dependency inventory and test matrix. Gate: recorded
   owner agreement on base, seed authority, accepted source, commit/reset/route
   transitions, API/memory and failure budget. Completed via relayed owner contract.
2. Numerical seed: implement cold-preserving initial-guess support and independent
   analytic/edge tests. Gate: same RHS-based thresholds, full true residual and
   compatibility/gauge checks, no hidden tolerance or iteration-budget changes.
3. Transactional integration: opt-in accepted seed and invalidation. Gate: rollback,
   late liquid/step failure, reset, route changes, failed retries, memory limits and
   default reference behavior. Use only owner-agreed files and base.
4. Performance qualification: pre-record primary and control workloads, then build
   isolated baseline/candidate binaries, retain every trial, reject adverse or
   ambiguous gains. Whole-step and physical-oracle gates in verification.md.
5. Independent review, final regression/format/lint and exact-head CI. Publish a
   separate draft PR only with truthful measured tradeoffs. No merge.

## Composed-design review — applicable, agreed

1. Numerical initialization owns residual/PCG algebra; state owner owns accepted
   fields, seed provenance and atomic publication; the per-step caller selects opt-in policy. These concerns are related but have separate authorities.
2. Required interleavings: accepted generation and operator identity qualify seed;
   publication happens with field commit. Accidental interleaving to avoid: solver
   success or scratch mutation changing cache validity before owner acceptance.
3. Callers should select cold/warm policy and supply budget only. They should not
   understand scratch lifetimes or manually commit a cache. Workspace receives a
   qualified immutable guess; it does not own authoritative accepted provenance.
4. A numerical initial-guess change touches solver/tests; reset/route policy touches
   state owner/tests; provider pressure provenance touches PR55 owner and the
   contract. Adding another warm domain is a new owner/invalidation qualification.
5. Stable inputs: geometry/density/domain identity, accepted generation, immutable
   pressure values. Hidden scratch representation or provider internals must not
   leak into identity. Runtime pointers and cell count alone are insufficient.
6. Cold solver and seed algebra can be verified independently; integration must
   verify atomic publication. Seed failure cannot publish fields or corrupt the
   last accepted cache. Cache can be removed without changing cold behavior.
7. Deleting the seed mechanism restores the cold path and removes its memory and
   provenance obligations. No general registry, adapter framework or new model
   provider is justified; do not retain one for hypothetical future domains.
8. Necessary complexity is accepted-state provenance plus one initial residual
   evaluation. Contain it at the state owner/solver boundary. The existing accepted array remains the only retained pressure copy; provenance is fixed-size owner metadata.

## Blockers and replan triggers

Replan if PR55 head/base changes, another owner must publish atomically, or a
new path changes the agreed ownership or execution budget.
Reject/revert implementation if correct seeds fail physical/rollback gates or
whole-step benefit is not repeatable against representative controls.

Standards: public coding-standards188beda1 Core/router, planning/discovery,
implementation, architecture/state authority, performance, verification/oracles,
Rust, commit and documentation. Plan admission follows actual owner evidence;
this document cannot self-authorize unresolved shared semantics.

See contract-proposal.md, verification.md, issues.md and execution-ledger.md.
