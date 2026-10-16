# Density change execution ledger

2026-10-16. Root is integrator. Work began from exact main
ad888bdbbbd5cae362439458af342196f51c58e2 in an isolated worktree, preserving other
agents' branches/worktrees. The user's revised requirement is 1000 cells/m,
10000 native cells/axis and independent preview edge up to 10000. The original
higher-density target is superseded. No Tuldok/surrogate-lab or user data changed.

Read adopted Core/Router, Rust/API/security, implementation, verification,
planning, build, commit, documentation and relevant resource/contract guidance.
Existing numerical, Session/PressureProvider, recording, worker and model support
owners remain authoritative. The canonical plan is [plan.md](plan.md).

Independent GPT-6.1 Sol review approved the exact source/test/document delta and
complete conservative admission closure. Its [report](review.md) and
[byte pins](approved-pins.json) remain immutable review evidence. Root corrected
the review's late 128² thumbnail failure, configured carrier/particle control
coupling, obsolete count clamp and unit-box reciprocal-roundoff compatibility
issue. Only the 14 explicitly approved selected-source hashes were rebound;
110 other selected sources and historical inventories remain unchanged. The
source-admission guard passes all 124 paths; it supplies no numerical receipt.

Expanded affected Rust checks passed on the reviewed working source:

| Cargo-selected target | Passed | Ignored |
| --- | ---: | ---: |
| rheon-desktop | 151 | 1 |
| rheon-ai-compare | 66 | 1 |
| cli_contract | 6 | 0 |
| geometry_contract | 8 | 0 |
| dense3d_sequence_contract | 8 | 0 |
| recording_carrier_contract | 4 | 0 |
| simulation_contract | 9 | 0 |

The ignored test is the same unavailable measured-deployment registration in
both binaries. Shared registrations and nested child summaries are not unique
additional tests. No measured A1 pass is claimed. Final strict all-target desktop
Clippy and formatting passed. The exact unit-box 49 compatibility correction
then received an independent source review and fresh CLI build; the release
comparison harness and subsequent committed-source receipts own that boundary.

Local logs are task-prefixed `/tmp/rheon-density-*`. The expanded Cargo command
uses `--locked --features desktop` with both desktop/comparison bins and the five
contract targets above. Cargo/rustup are pinned to the installed 1.92 toolchain;
matching artifacts in the existing idle comparison target were explicitly reused.
No empty-target, hermetic, authenticated or bit-reproducible build is claimed.
The comparison tool correctly refused the first debug-binary harness attempt;
Root preserved the failed log and selected a fresh task-owned release target
instead of bypassing the release guard. This is setup evidence, not numerical
failure. The separate source-stop numerical refusal remains in [issues.md](issues.md).

Final clean committed native/rational qualification, native controls, release
comparison checks and exact-head hosted/main acceptance remain separate gates.
Their source heads, tree identities, logs and outcomes belong to the linked PR
and external task receipts; earlier working-source checks are not relabelled as
final whole-Git evidence. Publication and merge are authorized after current
required checks pass. Retire only this task's clean branch/worktree after accepted
ancestry and dependency checks; preserve all other owners.

## Pre-merge preflight correction

The initial committed implementation was 94062f6e75cdbdd8828fcbe4e9dd3a4d01f1426f,
tree 877afa76b3728e15e00715799ced54c0e5e27bcb. Its clean native/rational
qualification passed 18 commands; its source-stamped software-window controls,
bit-identical saved numerical fields across two previews, release comparison
checks and hosted gates passed. Those external receipts retain that identity.
They are not whole-Git acceptance of a later head.

Before merge, audit found its comparison preflight used 2000 pressure iterations
while the existing engine permits 3000, and omitted independent mode's extra
same-request native solves. The correction moves the unchanged 3000 limit to
an engine-owned constant, reports a mode-specific total, and distinguishes the
core Simulation array-plan reference from actual diagnostic Engine allocation.
For 16³ cells, 64 steps and 15 warmup steps, legacy/teacher permit 64 native
solves (786432000 cell-iterations); independent mode permits 113
(1388544000). No pressure algorithm, threshold, model or physical semantics
change. The executable CLI regression covers all three modes without creating
output or executing a model.

The original source review/pins remain immutable. Exact follow-up review and
explicit source admission now cover 15 changed and 109 unchanged selected
paths out of the same 124. Fresh clean committed native/rational qualification
and all hosted checks must pass on the corrected head before merge. The earlier
GUI controls retain their bounded 94062 claim; the follow-up changes only
preflight metadata and ownership of an unchanged engine setting.
