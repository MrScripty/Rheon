# M2 native saved-intent contract handoff

Scope authority: user explicitly admitted M2–M3 implementation against
[PR84 plan f45cd91](https://github.com/MrScripty/Rheon/blob/f45cd91dab845c975fa6ab6952af02d5ba04d14b/docs/plans/configurable-solver-components/plan.md)
and its M0 reconciliation. This supersedes that proposal's earlier documentation-only
authorization for this delegated scope; it does not waive its numerical boundaries.
The integration lead owns M1, all shared composition and ALL merges.

## Exact ownership and disposition

Branch `feature/saved-runsets-m2-20261015`, worktree `/workspace/rheon-m2`, starts at
`0fbadbe733b5b521e192dc8a2aa980d6d69a1e01`. Only new app-owned
`src/bin/desktop/run_sets.rs`, `tests/saved_run_set_contract.rs`,
`docs/api/saved-run-sets.md` and this report are in the write set. No existing
source, exports, schema, Cargo/lock, workflow or other-agent work was changed.
The contracts are compiled through test path inclusion pending serial desktop
registration. They are draft implementation, not advertised production capability.

Retain this branch/worktree protected by its local and published branch while
review and dependent integration remain open. Owner: M2–M3 delegate; integration
and terminal retirement: lead. Keep it until all dependent branches no longer
need it and the lead records accepted/rejected/superseded disposition. No cleanup,
force removal, shared-history rewrite or independent merge is performed here.

Coordination is public on PR84 comments
[6198401466](https://github.com/MrScripty/Rheon/pull/84#issuecomment-6198401466)
and [6198560575](https://github.com/MrScripty/Rheon/pull/84#issuecomment-6198560575).
The selected environment is a fresh snapshot with no actual repository/ancestor
AGENTS or `.agents/skills`. The local Codex queue cannot reach the lead session
because the environment has no writable session database. Do not interpret an
unanswered coordination message as shared-source ownership approval.

## Implemented result

The [saved-run-set contract](../../../api/saved-run-sets.md) owns behavior and
limits. Native typed intent feeds existing Simulation; immutable manifests refer
to existing StepRecorder/RecordingReader archives. They do not own another store,
worker, numerical loop, runtime or restart mechanism. Full-intent declarations
bind trusted producer intent separately from actual archived fields/times/outcomes.
Cold reads never depend on current Debug rendering, current model code, live state
or a latest alias. Historical producer/build strings stay captured provenance.

Composed-design reconciliation against the plan:

1. Numerical acceptance, recording integrity, saved intent and analysis remain
   separate owners; this module owns application binding only.
2. Required coupling is exact immutable spec ↔ archive declaration and finalized
   artifact; display state and model channels do not enter the manifest.
3. Callers know the frozen spec, existing numerical API, protected directory and
   close-before-pin obligation. The composer applies the declared source schedule.
4. Native control additions change this codec/composer; archive evolution remains
   with recording; provider bindings require actual M1 types, not string guesses.
5. Only typed controls, identities and verified readers cross the boundary.
6. Missing/runtime-unavailable members cannot prevent cold numerical inspection;
   this implementation invokes no runtime. Storage errors retain their typed state.
7. Deleting the manifest loses immutable grouping/intent; deleting it does not
   remove a numerical archive or worker. No extra catalog/queue was retained.
8. Necessary version/hash/path/publication complexity is confined here; shared
   registration and real desktop composition are explicitly pending.

## Evidence and independent review

Adopted Coding-Standards source-guided route: exact `188beda1fa477d21d576c233dd8c7f4c4c267d23`
Core/Router; Implementation, Verification/Oracles, Development Proportionality,
Commit, Documentation; Library, Persistence, Rust/API/Security;
Architecture/Code Design/Replay, Contracts/Evolution, Security and Resilience.
No unsafe, new dependency, numerical method, process supervisor, supported platform,
release or graphical workflow is introduced. This is not an engine qualification.

Pinned Rust 1.92.0, locked dependencies, Linux CPU. Focused commands:

```sh
cargo test --locked --features desktop --test saved_run_set_contract
cargo clippy --locked --features desktop --test saved_run_set_contract -- -D warnings
cargo fmt --all --check
```

Independent GPT-6.1 Sol High review challenged full source/time binding, current
Debug/version dependence, generic native schema decoding, real process-exit
fidelity, actual budget consistency and accepted horizon checks. All were repaired
with focused regression cases. Producer declaration is explicitly trusted intent,
not independent proof of source execution/authenticity. Final frozen independent review found no remaining material findings. Its fresh
focused run passed 12 tests (0 failed/ignored), including the real producer-process
exit before cold-reader launch. Focused Clippy with warnings denied and workspace
formatting passed. Reviewed source SHA256:
`a16ee7bc09b51c7a805946a3e0b6c97fcee58c6398e1c728662b0a826082af0d`;
test SHA256:
`ba16503db127ad98a67b754a98ecb9a0ca07c16b970e1941f5dfc2524d91d5f5`.
Broader tests and exact draft/CI results are reported before integration.

The initial native fixture turned off its source after two steps and legitimately
hit the existing RHS compatibility gate on step three. It was replaced by a valid
three-step sourced fixture. No numerical gate or tolerance was weakened.

## Remaining boundaries

A3–A4 native automated portions have fresh evidence; accepted-provider A3 and
native desktop A4 remain pending. No accepted M1 production binding/physically
compatible composition was supplied to this worker, and shared source ownership
has not been agreed. Next dependent slice: lead supplies M1 outcome/provenance
contracts and allocates serial producer/UI registration; use those actual types
for provider recording, diagnostic schema, selected/fallback behavior and timings.
M3 comparison is a separate reviewed branch. Native GUI/model/lifecycle/other-OS
and full A0–A8 acceptance are not claimed. Existing archives are not restart
checkpoints and raw diagnostic JSON is untouched.

## Continued acceptance preparation after first draft

Parent directed continuation through the full M2–M3 assignment after initial
contract publication; these drafts are not the terminal milestone outcome.
Current lead source is diagnostic M1 draft PR87 at
`33a083a1fdf63c169c9bf68c71009d495d505f64`. It is main-based and deliberately
excludes unrelated training-viewer/voxel work from the older `0fbadbe` ancestry.
Only selected owned new files should enter its serial integration. Preserve the
original drafts and their exact source/review records; no ancestry-wide merge is
performed by this worker.

The next local acceptance delta exposes the existing private manifest I/O helpers
at application crate scope for the real saved-comparison consumer. The caller
supplies its expected schema version and practical byte budget; zero and
`u64::MAX` budgets reject before oversize-detection arithmetic/I/O. Typed payload
validation remains with each document owner. Path checks and streamed file
identity reuse the same owner; this adds no numerical store or public Rust API.

Four added tests use the existing numerical/recording owners:

- A real numerical step reaches accepted generation 2 after recording capacity
  admits only constructor and generation 1. The writer is poisoned, cannot claim
  completion, and records failed terminal status with the unrecorded accepted
  tail. A separately spawned cold reader observes that exact distinction;
  missing ordinal/unknown field stays missing.
- A closed pinned member's ordinal 1 stays bit-identical while a second real
  native track advances. This is in-process interleaving, not a threaded GUI
  interaction claim. The old run-set revision never acquires the later final pin.
- Real SQLite `BEGIN IMMEDIATE` contention makes terminal publication fail busy.
  The writer refuses retry, cold reopen stays active/incomplete, and pinning does
  not overwrite/delete the archive. This proves the sampled lock failure, not
  every possible uncertain commit outcome.

Focused tests now pass 16/16 with warnings-denied Clippy. Independent GPT-6.1 Sol
High review found no remaining material finding in this delta. Reviewed source
SHA256 `1e73a290ae1b96a7bb295372bb276c2c992e17ca3295075321cfef73f428b692`;
test SHA256 `d9c1e521256b5e8cbf3550726d7e882eb370069dae5667d0d9de628673b2d995`.
Shared producer/UI and source-inventory integration are still serial lead writes
pending its concrete allocation. Saved recipe/result implementation is continuing
against these actual helpers and the reviewed native comparison source.
