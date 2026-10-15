# M1 implementation admission and companion handoff

User authorization now includes M1–M3 implementation, separate draft PRs, validated
ready merges and deletion only of safely merged branches without dependent stacks.
GPT-6.1 Sol High only. Plan PR84 merged at `c85dfee`; main was clean and current.

Integration branch: `implementation/configurable-pressure-m1-20261015`.
Original comparison `6c38078` maps to selected cherry-pick `20b451d`; original
lifecycle `66e5038` maps to `d448563`. The unrelated training-viewer/voxel-renderer
commit `fe9d586` is excluded. Published stronger lifecycle assertions will be
retained at their exact owner, without importing unrelated local-agent edits.

## Ownership

Integration lead owns public pressure-context contract/export, existing
`desktop/ai_comparison/{engine,model_process,worker,mod,record,view}.rs`, root desktop
composition, Cargo/locks, existing `recording.rs`, API docs, plan and all merges.
Manifest subagent owns only new `desktop/ai_comparison/adapter.rs` with inline tests.
Independent reviewer has no implementation write authority.

Companion `01a140a3-ebca-7436-82cf-94fedc42b036` should prepare new persistence/
comparison modules and their focused tests on an isolated branch. Existing shared
files above are read-only until a serial handoff. Publish exact paths, base and
consumer contract before integration. Direct in-thread messaging cannot resolve
this separate-thread ID; this Git-backed document is the shared contract handoff.

## Consumer contract

Existing Scene and Frame remain the numerical source for diagnostic comparisons.
Scene fixes sealed viscous water, cubic cells, zero origin and fixed rho/mu/dt.
Frame owns native/AI pressure and tracer, quality, actual step/time and durations.
Its raw diagnostic states are not accepted Session publications. Saved run sets
must preserve mode, warmup, attempted/qualified/rejected counts and exact lineage.
Pressure/tracer fields alone cannot establish native staggered-velocity metrics.

M1 adds an additive public borrowed PressureContext under solver_provider, with
actual grid/rho/dt/interval, RHS and optional typed prior pressure lineage. Host
operator prepares initial residual. Accepted and diagnostic prior history have
distinct status. Existing v2 lab packages remain actual supported consumers.

New configurable trusted-local CPU package schema `rheon-pressure-adapter-v1`
references stage `sealed-mac-pressure-context-v1`. Named input bindings select RHS,
prior pressure, host residual, XYZ, density/dt from supported physical context;
adapter code owns tensors/channel normalization and can evolve without rebuild.
Output is declared full pressure or pressure correction, canonical cellzero gauge;
host reconstructs only the declared role. No manifest grants new physical support.
One existing ModelProcess/Worker remains the process lifetime owner.

M2/M3 recording integration must use the existing recording owner and a separately
typed diagnostic record path; never label failed raw AI as accepted archive v1.
Use versioned saved SimulationSpec/RunSet recipes and exact recorded physical-time,
geometry/stage/unit/gauge identity. Cold-open must not load model code. Preserve
raw JSON observations, missing/corrupt/incomplete states and opaque legacy metadata.
The production accepted-provider path remains conditional on separate physical
qualification; diagnostic-only M1 does not satisfy that prerequisite.

## Actual environment and gates

Rust 1.92.0 is available. Python and NumPy are available; Torch, measured deployment
and native X server/window environment were not observed. Do not claim their tests
passed. Actual subprocess protocol can be tested with independently authored CPU
adapters on controlled physical fixtures; that does not qualify the frozen trained
checkpoint or native interaction. Keep its negative accuracy observations.

Required before merge: scoped and affected Rust tests/Clippy/fmt, conformance and
process-failure checks, independent review and relevant CI. External/model/window
claims need their actual environment; no weakened checks or fabricated pass.
