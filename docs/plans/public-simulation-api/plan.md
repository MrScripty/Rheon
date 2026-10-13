# Public simulation API

Status: Blocked. Blocker: peer integration. Current phase: M3 contract coordination.
Objective acceptance: partial. Operation: continue this plan under the user's
explicit implementation/push/draft-PR instruction. Goal: embedded numerical data
and generation controls for Rust hosts, dataset consumers and game engines,
with separately evidenced physics backends rather than an invented universal solver.

## Authority and constraints

API worker owns `src/session.rs`, the additive `src/lib.rs` exports,
`tests/session_contract.rs`, `examples/{headless_session,game_loop_session}.rs`,
`examples/session_support/mod.rs`, `docs/api/simulation-session.md`, and this plan
folder. Physics, renderer, provider and quantitative algorithms remain with their
workers. Parent is the serial integration owner. Source base: `2209012`.
Feature branch `feat/public-simulation-api`, worktree `/workspace/rheon-api`, is
retained for draft review; preserve original `/workspace/Rheon` and owner changes.
No merge/deploy, history rewrite or resource cleanup is authorized by this slice.
After parent integration/rejection, disposition this worktree with verified refs.

Standards: public MrScripty/Coding-Standards at
`188beda1fa477d21d576c233dd8c7f4c4c267d23`. No local AGENTS or skills existed.
[Standards and design record](reports/design.md) documents applicability and probe.
Existing typed inputs, publication, dense3D wire bytes and fixed-case v2 capture
remain authoritative. The session owns lifecycle, never numerical acceptance.

## Milestones and claims

| Milestone | State | Claim and gate |
| --- | --- | --- |
| M1 transport contract and owner | Accepted | A1 public API accepts existing controls and preserves accepted publication on cancellation/rejection/deadline; A2 byte-compatible dense3D frame output; focused public contract tests and affected Rust regressions |
| M2 useful consumers | Accepted | A3 headless Rust consumer emits nine valid JSON frames; A4 engine-neutral host uses accepted dt, bounded catch-up and deadline; both examples executed |
| M3 shared backend integration | Blocked | A5 material/SPH, inspection and provider APIs expose supported capabilities and consistent state identity; needs peer contracts and combined source revision from parent |
| M4 actual Godot consumer | Deferred | A6 engine runtime proves conversion, thread/lifetime ownership and failure behavior; needs selected binding stack, project and supported Godot target |

A1/A2 are automated contract/integration evidence in Linux Rust 1.92.0, with no
special hardware requirement. A3/A4 are automated consumer integration evidence
in the same cloud CPU environment. A5 requires the real composed peer interfaces;
A6 requires a real Godot runtime/binding consumer. Rust examples do not satisfy A6.
A1/A2/A3/A4 satisfied in the execution ledger.
Exactly one next slice: obtain the parent's accepted combined peer-contract revision
to admit M3 implementation; M1/M2 are published in draft PR #48.

## Integration proposal for M3 (not implemented capability)

- Material/SPH: add a separately named material-session adapter around existing
  `ParticleFluidSimulation`, `ParticleFluidConfig`, `FluidParticle`, and
  `ParticleStepReport`. Reuse session run/source identity and publication stages;
  provide particle SI fields and actual dt, not a fake MAC grid projection. Physics
  owner must supply accepted control update/cancellation semantics before claiming
  transactional equivalence. API owner adds exports only after that gate.
- Quantitative inspection: inspected draft PR #47 at
  `1923d5bd3442a5369878a1a317b52f4a89026d5b`. Reuse its existing
  `ParticleDiagnostic` and `particle_diagnostics() -> Option<(f64, &[ParticleDiagnostic])>`;
  do not add a competing capture type. The slice aligns with `particles()` by
  append-only, run-local index; pair identity with `SessionIdentity.run_id`.
  Initial diagnostics are absent. Their committed evaluation stage is post-emission,
  pre-integration (and before wall response), distinct from accepted position/time.
  Preserve optional evaluation time, binary64 density kg/m^3, pressure Pa and
  acceleration m/s^2. Derived mass-times-acceleration excludes later wall impulses.
  Snapshot publication stage remains Constructor/Accepted; evaluation stage must
  be separately described rather than flattened into SnapshotStage. The PR adds
  ParticleDiagnostic near the existing particle exports; this branch appends session
  exports, so the proposed edits have separate regions. Parent integrates, no merge
  was performed. Inspection worker owns calculations and descriptor meaning. Consume immutable session state/grid/report plus run ID, carrier/liquid
  stamps, generation, accepted time, field units and stage. Inspection results must
  name their input publication and distinct pressure-interval semantics. Do not
  infer stress, force or end-surface pressure from transport snapshots.
- AI providers: provider owner supplies selection/capability/error types, model
  identity and inference budget/cancellation behavior. API accepts explicit selection
  only when the underlying backend can enforce it; retain native numerical acceptance
  and publish provider/model/version identity with accepted results. No hidden model
  fallback or claim that transport API already supports AI execution.
- CAD/UI consumes snapshots or copied display buffers; it does not mutate session
  state. Parent coordinates changes to shared exports and consumer contracts.
- Tuldok retains existing v1/v2 dataset owners. A generic controls sidecar is a new
  contract requiring consumer coordination, not an extension silently added here.

M3 write set must be revised once peer public types are known. Replan on changed
ownership, accepted-state semantics, wire evolution, backend capability, memory
contract, Godot stack, or real consumer evidence. No speculative backend registry,
common solver trait or cross-language ABI is included in M1.

Links: [ledger](execution-ledger.md), [issues](issues.md),
[contract](../../api/simulation-session.md), [design](reports/design.md).
