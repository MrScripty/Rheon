# Agreed owner boundary

Authority: PR55 owner's read-only review relayed by parent/user on 2026-10-13.
Policy: opt-in AcceptedPreviousV1, initially sealed constant-density numerical
solves. Existing entry points remain cold. Agreement authorizes evaluation, not
performance acceptance. PR55/57 and prior evidence remain unchanged.

1. Publication owner owns accepted seed; workspaces are attempt scratch. Reuse
   LiquidTransportSimulation's immutable accepted pressure during preparation.
   A standalone seed, if later supported, needs explicitly capped/reported memory.
   Lower solver borrows an initial pressure slice without inventing acceptance.
2. Publish only after all actual-f32 divergence, transport, reconstruction and
   final cancellation gates, in the infallible owner commit section. Every failed
   attempt preserves accepted pressure, provenance, snapshot, clock and quota.
   No fallible allocation/callback between owner swaps. PressureSession publishes
   qualified pressure, a different stage; no silent exchange of seed stages.
3. Identity includes owner/run incarnation, accepted generation, operator revision
   and stage. Operator identity includes geometry, density, boundary/topology,
   coefficients and gauge. Reset/replacement/import/operator or unsupported mode
   transition clears eligibility; a failed attempt does not invalidate history.
   Source/RHS/dt changes alone do not invalidate the fixed operator. Pressure stays
   in Pa, unscaled by timestep. Re-evaluate current stopping settings; do not carry
   residuals, directions or preconditioner recurrence across steps.
4. Preserve compensated full-RHS compatibility and budget; never mean-shift.
   Seeds must be finite, correctly sized and gauge-zero. Compute full b-Ap,
   including gauge equation, before convergence/PCG initialization. Preserve true
   residual, predicted and actual divergence gates. Relative tolerance uses the
   current RHS norm. Zero iterations require every applicable gate.
5. Preferred providers and their numerical fallback remain cold. Successful
   provider publication invalidates native seed; failed attempts preserve it.
   Never seed from rejected/partial/nonfinite learned output. Cancellation aborts.
   No automatic warm-to-cold retry initially; any later retry must explicitly
   bound/report total work. Existing provider equivalence/replay stays intact.
6. Accepted publication reports requested policy, actual initialization, seed
   generation/stage/operator revision and cold reason; provider identity remains
   separate. Rejected attempts cannot replace snapshot metadata. Advertise API
   capability/version. Existing dense3D does not record warm history.
7. Test cold-default compatibility, warm reuse, identity/reset/invalidation,
   compatibility/gauge/zero RHS, downstream cancellation/retry, memory limits and
   unchanged provider/fallback. Equal physical accuracy, not cold/warm bit identity,
   is required. Replay requires same policy/history/accepted inputs. Benchmark
   whole accepted steps including seed/residual costs and adverse cases.

Implementation selection within this contract: coupled LiquidTransportSimulation
and its SimulationSession facade first, reusing accepted pressure. Standalone
Simulation warm storage is deferred (ordinary Simulation stays cold). Stack on
frozen PR55 head3525c979 on a separate branch; no owner branch edits. Unsupported
surface/column/obstacle/boundary/viscosity/provider paths remain cold and invalidate
native eligibility only when they commit. No contract conflict identified yet.
