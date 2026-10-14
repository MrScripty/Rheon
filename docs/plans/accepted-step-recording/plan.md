# Per-step numerical recording and aligned inspection

Plan status: Accepted. Current phase: M1R independent-review repair acceptance.
Next slice: none. Acceptance status: M1 and M1R satisfied; extended coverage has a separate admission.
[Ledger](execution-ledger.md) · [Issues](issues.md) · [Design](reports/design.md)

## Objective and authority

Retain every accepted material step on disk and inspect/export exactly the selected
recorded timestep after the producer has ended. User authorized implementation,
commit/push and PR creation (Sentinel_612c1a025b94819195f656e234bfd26b).
Canonical admission: this plan, operation start; this task owns the recording slice.
Standards source: MrScripty/Coding-Standards `188beda1fa477d21d576c233dd8c7f4c4c267d23`.
Core/Router, planning/discovery/development-proportionality, implementation/build, verification/oracles/GUI,
commit/documentation, library, persistence, Rust/API/security/dependencies,
architecture/replay/code-design, contracts/evolution, concurrency, diagnostics,
performance, resilience, licensing and cross-platform apply. Frontend web profile is excluded: this is native egui with no web UI responsibility. Concurrent plan-integration protocol is excluded: no two workers update this canonical plan. No standards-engine certification.

## Scope, constraints and assumptions

CPU native recording, bounded record payload and SQLite storage, indexed cold
reload, exact numeric blobs, metadata, selected inspector and existing VTK contract.
Generic named-field archive supports current independent numerical consumers.
The material desktop is the first integrated producer. Other solver UI producers
must explicitly disclose absent recording. No physics/GPU/remote feature, restart
checkpoint, main merge or changes to PR67. Solver configuration/control identities
are authored provenance, not proof of solver reproducibility or restart capability.
SQLite paths are local, caller-owned and protected from concurrent directory/path
replacement; untrusted/shared filesystem mutation is unsupported. Single writer.

## Binding decisions (owner: recording task)

SQLite atomic transactions own step publication, indexed ordinal/generation lookup
and crash rollback; DELETE journal/FULL sync, no WAL retention or writer resume.
Pinned established SQLite adapter and SHA256 implementation replace custom journals
and cryptography. Binary LE blobs retain IEEE754 bits. Schema version 1 rejects other
versions; no migration or latest-value fallback. Keep all committed steps; stop at
explicit limits; no eviction or automatic deletion. Unknown active terminal status
on reopen discloses crash/incomplete run and possible unrecorded accepted tail.
Recording failure is distinct from physics failure: a just-accepted unrecorded step
is disclosed, recording stops, earlier selected publications remain authoritative.

## Objective acceptance

| ID | Observable criterion | Kind / environment / mode | Status | Evidence |
|---|---|---|---|---|
| A1 | Exact particle/diagnostic round trips and selected historical values, times/units/identity | contract / Linux CPU / automated | satisfied | [verification](reports/verification.md) |
| A2 | Rejected/cancelled physics steps add no record; recorder rollback and cold reload | integration / Linux CPU / automated | satisfied | [verification](reports/verification.md) |
| A3 | Storage/payload/step caps, corruption, version rejection and incomplete run disclosure | negative contract / Linux CPU / automated | satisfied | [verification](reports/verification.md) |
| A4 | Inspector/particle view/export follow selected timestep; missing step gives unavailable | UI state / egui headless / automated | satisfied | [verification](reports/verification.md) |
| A5 | Pinned fmt, strict Clippy and affected regression matrix | regression / Rust 1.92 Linux / automated | satisfied | [verification](reports/verification.md) |

Native window interaction remains environment-qualified evidence, not a CPU contract.

## Simplicity and ownership review

Applicability: applicable. Numerical snapshots remain canonical field producers;
recording owns persistence, the inspector owns selection, VTK owns interchange.
State/identity/time are packaged together per transaction so selection cannot mix
latest diagnostics with historical positions. Schema version owns only archive
layout; material/API/VTK contract versions remain independent referenced authorities.
Callers supply identity, actual configuration/geometry/controls, limits and a local
owned path. They need no index/journal/checksum ordering knowledge. Adding a numerical
field changes its adapter and contract tests; storage policy changes only archive;
selection changes inspector; physical semantics remain with solver/API owners.
Native-only persistence can fail without changing numerical acceptance, with explicit
terminal disclosure. Existing VTK consumes selected decoded canonical types. Deleting
archive redistributes transactions, bounds, integrity and indexed reads across UI
and consumers; retaining it contains necessary complexity. No backend registry,
queue, checkpoint codec, eviction, migration or remote service is introduced.

## Milestones

M1 goal: runnable CPU-first archive and aligned material desktop.
Allowed writes: `src/recording.rs`, `src/recording/*`, additive `src/lib.rs`,
`Cargo.toml`, `Cargo.lock`, `browser/solver/Cargo.lock`, `src/bin/desktop/{inspector,history,particle_view}.rs`,
`src/bin/rheon-desktop.rs`, `tests/recording_contract.rs`,
`examples/recorded_material.rs`, `docs/api/recording.md`, this plan directory.
Gate: A1–A5 satisfied. Status: Accepted.

M1R goal: resolve independently reproduced reliability defects before expanding producers.
Allowed writes: `src/recording.rs`, `src/recording/material.rs`,
`tests/recording_contract.rs`, `docs/api/recording.md`, this plan directory.
Admission: user-authorized continuation, operation start, existing feature branch/PR69.
Status: Accepted, operation verify, independent re-review resolved all three findings.
Gate: real SQLite busy terminal attempt blocks further publication; poisoned writers
cannot claim completion; every declared terminal accepted generation covers the
durable recorded tail; constructor/accepted material diagnostic inventories match
their canonical contracts. Negative fixtures must fail at the named boundary.
Independent review: read-only GPT-6.1 Sol High agent reproduced all three defects;
existing 11 archive tests passed and therefore did not cover these negative cases.
After M1R, separately admit measured storage decision and carrier/cold desktop slice.

## Evidence plan and findings audit

Independent oracles are solver-owned snapshots and hand-decoded LE bits; cold
process example/Python SQLite consumer observes emitted data, not a mock store.
Adversarial records alter committed blobs/metadata and truncate files; failures
must distinguish integrity, limits, unsupported version and missing selection.
Consumer inventory: desktop inspector/point view/VTK share selected canonical
material values; appearance preview retains its separate lower-resolution contract;
other solver modes display recording unavailable. No selected data uses ambient sim.

## Isolation and coordination

Admitted PR67 `24b38606e8c8354e56a2f83f1d0c44d6cd7b932e`; inspected main
`c7e88d8dfd58969e6020d089a11fb96795f0f5e3` (final character corrected in ledger).
Branch `feature/accepted-step-recording` in existing `/workspace/Rheon`, retained for
PR review, no task-created worktree. Parent/PR67 sole merge owner owns serial
integration, prerequisite qualification and eventual disposition. This task may
push only its feature branch and open dependent PR. PR67 branch remains unchanged.
No compressible-flow research overlap or sub-agent delegation.

## Blockers and re-plan triggers

No implementation blocker. Replan for unavailable SQLite transaction semantics,
material identity change, integrator/restart requests, other producer integration,
shared filesystem mutation, retention eviction or source-qualification ownership.
Final acceptance: satisfied for M1. Carrier integration, cold desktop opening, native-window review and fresh educational qualification remain explicit follow-ups owned by parent/integration or their qualified owner, triggered by the next requested scope or merge admission. [Final evidence](reports/verification.md). No outstanding implementation within M1.
