# PR146 CI diagnosis and item-order repair

## Verified subject and observations

Fetched main7516a8e5d328c6dc8eda8ea577cb03f20b2e343a / tree
fef62baaeec871ff1d7d7083f73ab843a9cf93df without merging/rebasing or changing the
UI source subject9099b150e8b80cd750db08276c51e180be4e61f6. PR146 remains a draft
against main. PR145/backend source and admissions have distinct preserved subjects.
The3b sealed Worker cases already passed once; no new numerical test was run here.

Hosted job118964260530 (1b377742) and current118972927185 (9099b15) failed
**Strict lints for selected features**, before desktop contract test execution.
Their annotations give exit101, not compiler details. The log-download routes
returned network Forbidden; downloads were stopped. Do not present local output as
a retrieved hosted compiler log. No workflow rerun or gate weakening was requested.

Local compile-only reproduction on9099b15:
`cargo clippy --locked --offline --features desktop --bin rheon-desktop --tests -- -D warnings`
failed with two `items_after_test_module` diagnostics: the cfg(test) native
presentation main after the desktop tests; Inspector bind_force_fields and
verify_force_fields after its tests. The repair literally relocates those unchanged
blocks before their respective test modules. No lint suppression, function-body,
visibility, cfg, dispatch or custody change. The same compile-only strict command
then passed in27.13s; formatting and diff checks pass. No test bodies, Worker,
SQLite or native session executed. Full hosted CI acceptance remains pending.

## Book admission remains correctly refused

Current bookbuild job118972927128 stopped at **Check reviewed native admission
before expensive qualification**, exit1. No numerical comparison ran in that job.
Raw stderr is unavailable because its log download was Forbidden. Read-only source
comparison independently confirms a stale inventory: branch9099b15 retains the
historical124-path review/inventory; its conservative source closure is136paths,
12added/18changed/none removed. Merged backend main has its own126-path admission,
which does not cover the later desktop closure. This branch has not merged that
admission; preserving it alone would still leave added/changed UI sources unadmitted.

The local full admission script cannot reach the inventory guard because this
checkout is shallow and lacks immutable primaryf75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b.
Its preserved traceback is an environment limitation, not a reproduction of hosted
stderr. The inventory diagnosis separately compares the existing selector's whole
set and hashes without asserting ancestry/qualification. No inventory, review record,
selector, threshold, workflow or source pins changed. Never refresh these hashes
automatically or copy backend receipts as UI acceptance. A later coordinated source
integration needs explicit complete desktop-closure review/admission and fresh exact
committed native/rational/publication evidence under that owner's authorization.

## Evidence and remaining boundaries

[Exact logs, metadata, source seals and hash manifest](../evidence/ci-item-order-20261021/manifest.json)
retain the initial lint failure, repaired strict result and shallow-history refusal.
[Independent relocation review](ci-item-order-source-review.md) covers source semantics.
The policy stays Pending. Native gesture/force-launch/cancellation acceptance,
full CI and book source admission remain open. No new execution budget, security,
network configuration, merge or cleanup is part of this repair.
