# AI pressure provider milestone

Goal: choose real learned pressure inference alongside Rheon's numerical references,
with explicit scope, qualification, provenance and fallback. Base: remote main
2209012bc5f902693d0f695ec1c2d854126af317. No merge or deployment.

Repository instructions: no AGENTS.md or .agents/skills found in the checkout or
workspace parents. Apply docs/REPOSITORY_OUTPUTS.md and rust-rheon.yml CI checks.
Preserve the standard-library-only core and existing numerical APIs.

1. Inspect lab contracts and checkpoint. Accept only the frozen trajectory-v2
   checkpoint, retain source/hash/licensing identities, and train nothing.
2. Add isolated public solver_provider module, fixed-box pressure request,
   extensible descriptive tensor/stage/version metadata, native CPU inference,
   rejection diagnostics and deterministic numerical fallback. No wire protocol,
   desktop settings or shared simulation edits. API owner can wrap this module.
3. Acceptance: both numerical choices remain available; malformed input fails
   before inference; unsupported model domains and invalid predictions fall back;
   gauge, full residual (including cell zero), finite values and predicted
   divergence pass identical requested tolerances. Cancellation aborts execution.
   Candidate storage is never accepted simulation state. Actual f32 corrected
   velocity divergence is separately tested by the executable comparison.
4. Compare actual learned weights against both numerical references on CPU,
   including cold artifact loading and complete provider invocation timings;
   reproduce model output independently from the original artifact. Preserve
   measured scope, runtime, source and hardware evidence in ignored output.
5. Independent review, strict formatting/lints, core/default/parallel tests,
   commit/push dedicated branch and draft PR. Document unsupported hardware and
   physics, remaining qualification limits and exact commands.

Owner write set: src/solver_provider.rs, src/solver_provider/*, one module export
in src/lib.rs, tests/solver_provider_contract.rs, examples/ai_pressure.rs,
tools/*ai_pressure*, docs/implementation/ai-provider/*, and a dedicated
.github/workflows/ai-pressure.yml. No existing shared workflow edits.
Shared contract proposal sent in task commentary for parent relay. The provider
stage API works directly with a sealed RHS and leaves velocity correction and
transactional simulation ownership to the caller; it cannot accept free-surface
or obstacle operators accidentally.
