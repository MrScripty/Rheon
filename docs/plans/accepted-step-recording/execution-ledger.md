# Execution ledger

## 2026-10-13 — start and discovery

User-authorized operation start, canonical plan docs/plans/accepted-step-recording/plan.md.
Read pinned Core/Router and selected workflow/profile/topic owners. Existing standards
indexes are non-normative; used canonical owners. Main c7e88d8dfd58969e6020d089a11fb96795f0f5e3
and PR67 24b38606e8c8354e56a2f83f1d0c44d6cd7b932e fetched/inspected. Correct main OID above
supersedes transcription typo in initial plan. No AGENTS.md in selected workspace.
Pinned Rust 1.92.0 already available through /workspace/rheon-setup/env.sh. Native
SQLite pkg-config unavailable; explicit bundled-library dependency planned. Original
checkout clean, separate dependent feature branch created, PR67/main untouched.

## 2026-10-14 — implementation and verification

Implemented native SQLite atomic step/field publication, fixed keep-all/stop policy,
explicit page/payload/record caps, binary field bits, run/configuration/geometry/control
provenance, SHA256 integrity, indexed selection and cold reload. MaterialSimulation
and MaterialSnapshot adapters use canonical capture; VTK uses the existing contract.
The worker records every accepted material step before its preview; selection, points,
CSV and VTK follow that exact recorded timestep. Missing data clears the selected
view. Rejected/cancelled candidates create no step. An accepted but unconfirmed write
is disclosed; append admission stops and a fresh reader reconciles the durable prefix.
Header identity is rechecked on selection/audit; audit uses one read transaction.
Selected-particle callbacks retain one immutable publication and release the mutex.
Absent historical particle IDs remain absent rather than clamping to another ID.

Root dependency changes also invalidated the browser's nested lock. Expanded the
owned write set to that lock only, resolved it minimally, checked all prior identities
unchanged, and passed locked WASM with recording deps excluded. Source/physics/browser
algorithms, PR67/main and pinned educational admission records remain unchanged.
Compared native-only storage candidates before choosing pinned rusqlite 0.37.0
(bundled libsqlite3-sys 0.35.0) and sha2 0.10.9; Cargo source/checksums and authoritative
crate license texts were inspected. Rusqlite/libsqlite adapter MIT, SHA2 MIT/Apache2,
SQLite public-domain dedication. No vendor code or binary distribution added; future
binary packaging must carry applicable dependency notices, owned by release.

Plan operation verify. [Verification report](reports/verification.md) maps A1–A5 to
exact commands, outcomes and evidence limits. Initial mutation-test expectation and
concurrent desktop timing failure are retained there; final focused/isolated checks
replace them without weakened production gates. Full core regression passed 437;
three existing checkpoint tests remained ignored. Generated numerical data/logs are
not staged. No real-model, native-window, research-book qualification, GPU or restart
capability claim. Feature ref remains protected for dependent draft review, with
parent-owned serial integration/disposition. No task-created worktree or cleanup.

Final source checks passed: core/default archive 11 each (none ignored), desktop 36
in isolation (none ignored), strict Clippy in core/default/desktop, formatting and
diff checks. Final point UI labels identify selected data, CSV includes run ID,
and the real egui test preserves a particle ID absent from an older publication.
A1–A5 are satisfied for the admitted M1 scope. The associated atomic commit is
`feat(recording): retain accepted material steps and align inspection`; its write
set is the archive/adapters, desktop integration/tests/example/contracts/plan and
native dependency declarations/root+nested locks. Source-only commit: generated
fields, numerical outputs, logs and temporary tools are excluded. Publication uses
only the protected feature ref and a dependent draft; the PR/final handoff carries
the actual commit/PR identity. PR67/main integration remains solely parent-owned.

## 2026-10-14 — M1R independent review repair

Operation start/continue/verify: user explicitly authorized independent review and
three reproduced fixes before further coverage. Read-only GPT-6.1 Sol High reviewer
reproduced busy terminal admission, contradictory valid-hash terminal tail and
canonical stage/inventory defects. Repairs distinguish Open/Poisoned/TerminalAttempted,
close admission before fallible terminal reconciliation, refuse poisoned completion,
and validate declared accepted generations against durable recorded tail in writer
and cold audit. Canonical material decoding enforces constructor/accepted inventory;
generic sparse records remain accepted and absent fields remain absent.

Negative regression coverage uses real SQLite shared/exclusive locks, poisoned cached
tail, both failed/cancelled outcomes with lower/absent generation and independently
computed correct hashes, and both material stage variants plus sparse generic reload.
Archive tests core/default: 15 passed each. Isolated serial desktop: 36 passed.
Independent re-review: all three findings resolved; an additional exclusive-lock proof
confirmed admission closes even if the tail query itself fails, before UPDATE.
These tests qualify observed SQLite failures, not exhaustive power-loss outcomes.

Current original-head PR69 CI has passed numerical, VTK, cloud and bounded lessons;
feature matrices were still running and book build failed. Local unchanged source
admission gate independently refuses the added recording source paths. The admitted
runtime inventory is missing recording.rs and recording/material.rs and has six
changed crate/desktop inputs. Posted exact evidence and serial-owner handoff in
PR69 issuecomment-6158642258; no inventory/gate/receipt or PR67 source changed.
Forbidden remote failure logs were not bypassed. Publication owner owns new committed
source admission and fresh qualification; this task does not claim book acceptance.
