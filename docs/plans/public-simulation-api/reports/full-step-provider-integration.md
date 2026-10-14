# M6: production full-step provider integration

Status: implemented and independently reviewed; local verification passed, external validation handoff pending.
Authority: parent explicitly requested production Simulation/Session selection rather
than another standalone harness. Start from PR53 `99bd039`; feature branch
`feat/full-step-provider-api` merges complete PR49 `89997c5` (merge `e713854`), including
its independently strengthened trajectory transition and receipt-bound checks.
Original worktree and published PR48/PR53 remain unchanged.

## Contract and write boundary

API owner adds caller-capped `ProviderStepWorkspace`, additive production step methods
and accepted session metadata. The integration edits `simulation.rs`, `liquid_step.rs`
and `session.rs` only at composition/publication boundaries; no provider implementation,
pressure operator, numerical algorithm, renderer or AI harness is forked. Shared
provider trait/request/report remain unchanged. Provider metadata is attached only
at the already accepted full-step boundary. [Shipped contract](../../../api/full-step-provider.md).

The existing numerical prepare/commit split is the authority. The sealed RHS is built
from the same force/advection candidate. Stored f32 divergence and Courant, tracer,
liquid volume, finite advancing clock and checked generation gates cannot be bypassed.
Boundary/free-surface/column/viscosity composition is outside this bounded slice.

## Milestones and acceptance criteria

1. Contract/design: explicit synchronous ownership, sealed admission, actual-dt model
   support, array caps, pressure-only fallback and all-or-nothing publication.
2. Implementation: production Simulation, liquid facade and Session calls; existing
   legacy paths retain results/stage sequences; accepted pressure uses selected scratch.
3. Automated evidence: exact legacy/numerical full-state comparison; real checkpoint
   selected over 24 complete intervals; actual stored-field divergence independently
   recomputed; fallback, every observed cancellation stage, injected deadline crossing,
   post-pressure velocity/liquid rejection, unsupported modes, identity and cap refusal.
4. Independent review: readonly API review plus parent-coordinated validation task
   `01a134aa-fa83-76e1-8e5d-87a010e33017` against the pushed revision.
5. Delivery: strict lints, feature-appropriate regression suites, working example,
   committed/pushed stacked draft; no main merge/deploy or hardware claims.

## Design probes

- Pressure qualification is insufficient: a zero-output probe with permissive pressure
  settings must still fail the production stored-f32 divergence gate.
- A liquid failure after accepted carrier preparation must leave both owners, accepted
  pressure and provider metadata intact, then permit an ordinary retry.
- No inference report is published merely because the dispatcher returned success.
- Separate provider array accounting is explicit; retained simulation/session scratch
  remains counted, model memory is caller-owned, and no process cap is inferred.
- The existing provider can mutate itself on a failed call; only the production owner's
  state transaction is promised. Callback/kernel execution cannot be preempted.
- Retain final AI verifier code from `89997c5`; do not transfer its independent harness
  numerical claims to this wrapper without running production-path tests.

## Current evidence

Initial real-artifact focused run: 5 tests passed, including all 24 model-selected
production intervals. Independent reviewer repeated those 5 and 34 affected legacy
regressions with no blocking findings. Follow-up adds its suggested domain/workspace
negative tests, post-pressure liquid rejection and explicit provider-byte breakdown.
Final counts/commands and limitations are recorded in the execution ledger.
