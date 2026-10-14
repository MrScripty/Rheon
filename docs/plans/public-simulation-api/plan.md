# Public simulation API

Status: Accepted (M1–M3 and M6 implementation scope). Current phase: independent validation handoff.
Objective acceptance: satisfied for M1–M3; M4/M5 deferred; M6 implementation/review accepted. Operation: verify under the user's explicit
implementation/push/draft-PR instruction. Goal: embedded numerical data and
generation controls for Rust hosts, dataset consumers and game engines, with
separately evidenced backend capabilities rather than a universal solver claim.

## Authority and constraints

API owner writes session modules (`src/{session,material_session,pressure_session}.rs`),
additive `src/lib.rs` exports, matching session contract tests/examples, shared example
fixture, `docs/api/` contracts and this plan folder. Physics, diagnostic algorithms,
providers and CAD rendering/VTK remain with their respective workers. API owner is
responsible for integration design; parent owns eventual main integration.

Source base `2209012`. PR #48 at `f06a127` remains unchanged. The current branch
`feat/material-simulation-api` in `/workspace/rheon-api` stacks on it and merges the
complete published PR #47 (`1923d5b`) and PR #49 (`b8ca87c`) dependencies, preserving
history. Their only conflicting source region was additive `src/lib.rs` declarations;
both were retained. No main merge/deploy or history rewrite. Original `/workspace/Rheon`
remains preserved. Retain this worktree protected by the feature ref for draft review;
parent owns integration/rejection and then its disposition with verified reachability.

Standards: MrScripty/Coding-Standards at `188beda1fa477d21d576c233dd8c7f4c4c267d23`.
No local AGENTS/skills existed. [Design record](reports/design.md) contains applicability
and current composed-design probes. Existing numerical acceptance and dense3D/v2
contracts remain authoritative; the API owns lifecycle/publication, not algorithms.

## Milestones and acceptance

| Milestone | State | Claim and deciding evidence |
| --- | --- | --- |
| M1 transport API | Accepted | A1 controls/lifecycle and rollback; A2 existing dense3D bytes; public contracts and native regressions |
| M2 embedded consumers | Accepted | A3 nine valid headless frames; A4 bounded game-loop consumption of actual dt; executable examples |
| M3 material and provider composition | Accepted | A5 native-equivalent material fields, canonical diagnostics and retained controls; A6 qualified pressure selection/report, explicit fallback and prior-publication retention; native and frozen-artifact contract tests |
| M6 production full-step provider | Accepted; external validation pending | A9 existing full-step gates and rollback, real checkpoint trajectory, caller caps and accepted metadata; [integration plan](reports/full-step-provider-integration.md) |
| M4 actual Godot binding | Deferred | A7 actual engine conversion/thread/lifetime/error behavior; needs a selected binding stack, Godot project and runtime acceptance |
| M5 tangential moving-plate session | Deferred | A8 layered speeds, interval reports and atomic clock/control publication; separately gated experimental-library exposure, native equivalence and dense-rational evidence |

A1–A6 are automated contract/integration evidence on Linux CPU with Rust 1.92.0.
A6 requires the real SHA256-pinned frozen export and observes actual provider calls,
not test-double evidence. A7 requires a real Godot consumer; Rust examples do not
satisfy it. M1/M2 evidence and exact command/configuration counts live in the ledger.
Next slice: parent-coordinated independent validation against the pushed M6 draft. Parent reviews the draft;
M4 needs a separately selected binding/runtime acceptance path. M5 has a concrete
[separate integration contract](reports/moving-plate-integration.md), reviewed against
PR #52 `fd281eb`; it is not advertised by the current session capabilities.

## Binding integration decisions

- `MaterialSession` owns existing SPH configuration, per-call emitter controls and
  borrowed snapshots. Configuration is fixed for a session; successful reports use
  actual accepted dt. Rejected requests cannot relabel accepted controls.
- PR #47 owns `ParticleDiagnostic` and `particle_diagnostics()`. API reuses that exact
  capture. Evaluation is post-emission/pre-integration, before wall response; accepted
  positions/time are later. IDs pair caller run ID with append-only particle index.
- `PressureSession` uses PR #49 `PressureProvider`, metadata/provenance and qualification/
  fallback reports. Its output is explicitly **qualified pressure-stage** data. The
  separate M6 route composes the actual velocity correction/divergence/commit path
  without bypassing its acceptance gates.
- CAD PR #50 (`3f49a15`) owns VTK, export UI and live-field rendering. Raw API views are
  its input. No competing VTK/renderer or diagnostic capture is introduced.
- Tuldok dataset completion remains with Dense3dSequence and existing v1/v2 exporters.
  Example particle CSV and standalone pressure data are not relabelled as v2 datasets.

The material and pressure APIs deliberately have separate result types because
accepted particles, diagnostic evaluation and qualified projection pressure have
different physical semantics. No backend registry, universal solver trait, network
service or ABI is introduced. M6 now composes the physics-owned candidate/correction/
actual-divergence/commit path, with separate rollback and real-model evidence.
Replan for changed ownership, stage meaning, wire contracts, backend domain, memory
promises or selected Godot binding. Ordinary full-step verification repairs remain in M6.

Links: [ledger](execution-ledger.md), [issues](issues.md), [design](reports/design.md),
[transport](../../api/simulation-session.md), [material](../../api/material-session.md),
[pressure](../../api/pressure-session.md).

M5 was selected as a separate milestone because PR #52 is a path-import experiment
with no clock owner and a layered tangential state incompatible with current session
models. API owns its integration design; physics owns interval/numerical admission.
No PR #52 source was merged or unsupported general-geometry capability added here.

M6 expands API-owned writes to composition boundaries in `simulation.rs` and
`liquid_step.rs`, the new provider-step workspace, matching tests/example and
contracts. Numerical algorithms and AI provider implementations remain unchanged.
