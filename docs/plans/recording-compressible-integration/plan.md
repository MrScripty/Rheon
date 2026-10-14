# Plan: recording and periodic S1 integration

**Plan status:** `Verifying`

**Current phase:** qualified source integrated; checkpoint/evidence handoff

**Next slice:** final metadata-only checkpoint-document checks and ready-PR disposition

**Acceptance status:** `partial`

**Execution ledger:** [execution-ledger.md](execution-ledger.md)

**Issues:** [issues.md](issues.md)

## Objective

Compose reviewed recording, carrier inspection, independent validation and periodic
S1 proposals on reviewed main without changing their refs or losing history.
Qualify the actual combined source before authoritative integration. Parent
assigns sole serial integration ownership, ready merges and qualified book
publication; the continuation authorizes the canonical S1 checkpoint only after
combined-source gates pass. No S2 or later implementation starts here.

Original admitted main is `06eb5daee9e77e2b0843a5ee0a4105f6950e9b1b`; prospective
plan is `fc2270a85e38080d9943048aaf689d99545d3715`. Pinned proposals: recording69
`b1acf2e514b88b528cf43e873035a598a1080ae1`, carrier71
`ecfc741af733f909c79170530080a98fdc5c78eb`, validation73
`27193c77b173a7437494584438847fedc8ecf552`, native74 numerical subject
`5a049ed604b99817a428d10bbc0140732d3421d6` and documentation-only final handoff
`a0efbc1aaf67bb65a724de028d4a773270977875`. Trial merges preserve every ancestor.
Compare current authority/OIDs before shared writes and merge. Moving heads need
source comparison and fresh admission, never automatic latest selection or rebase.

## Objective Acceptance

| ID | Observable criterion | Kind | Environment | Mode | Status | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| R | Material/carrier accepted-step durability, precise historical/cold selection/export, limits/integrity/crash/failure disclosure and actual native reopen/select/export | contract + user-workflow | CPU; required-real native window, software rendering sufficient for functionality | either | satisfied | [Focused checks](reports/verification.md); [Native software display](reports/native-display.md) |
| C | Periodic first-order ideal-gas S1 public API/rollback and actual prospective fields meet unchanged gates; original coarse failures stay failed | contract + numerical integration | pinned Rust CPU/NumPy | automated | satisfied | [Native handoff](../cpu-compressible-solver/reports/implementation.md); [fresh capture](reports/verification.md) |
| B | Complete source review, preserved historical inventories, fresh clean native/rational/packet and full final-head book/browser/runtime qualification | contract + numerical integration + system | CPU; required-real hosted Chromium/book build | automated | satisfied | [Admission review](../../education/NATIVE_ADMISSION_INTEGRATION_REVIEW.md); full run38532464608/job115747322162 passed on4bcdb7f |
| D | Reviewed plan, exact-head terminal CI and gate-qualified PR disposition preserve refs/authority, with checkpoint and limitations recorded | release-artifact + contract | required-real GitHub | either | pending | [PR76](https://github.com/MrScripty/Rheon/pull/76) merged58d647d; metadata follow-up disposition pending |

## Scope and constraints

Coding-Standards `188beda1fa477d21d576c233dd8c7f4c4c267d23` Core/Router,
planning/discovery, implementation/build/verification/oracles/GUI, commit/docs,
Rust/API/library/persistence, security/dependencies/licensing and architecture/
contracts/concurrency/resilience/performance/cross-platform owners apply.
Concurrent-plan-integration applies to outstanding proposals and intervening
canonical-plan/shared-export changes; explicit Git OIDs are the revision mechanism.
All workers/reviews use GPT-6.1 Sol High. No full Engine certificate.

No S2, S4 compressible session/recording adapter, new physics, GPU, remote,
training/surrogate/vehicle work, numerical gate relaxation, universal-field or
restart claim. Software display functionality is separate from hardware/frame-time/
RSS/performance evidence. Generated outputs stay external.

## Binding Decisions

| Decision | Owner | Evidence/consequence | Supersedes |
| --- | --- | --- | --- |
| Preserve lineage through isolated trial merges and additive exports | serial integration | Only conflict was src/lib.rs; component source files remain identical | none |
| Retain newer canonical main plan | parent/integration | Common-ancestor merges preserve fc2270a; compare OIDs before authority changes | stale branch-plan admission |
| Keep recording, numerics, verifier, source and publication acceptance separate | respective owners | Projection gauge pressure is never EOS pressure; old receipts retain subjects | none |
| Explicit reviewed v3 epoch; preserve both earlier inventories exactly | integration/education | Full 78-path closure and predecessor/review/committed receipt bindings; no fallback/refresh | earlier epoch's future admission scope only |
| Minimal standalone lock repair, root-aligned new identities | integration; original validator authority retained | 49 identities/checksums preserved, 22 additions/license texts reviewed | stale combined-graph lock admission |
| Authenticated official user-space Xvfb and llvmpipe functionality smoke | runtime/integration | Actual native carrier/material cold inspection/export passes; no browser/security/global-install changes | initial missing-display blocker |
| Canonical S1 checkpoint only after combined-source gates pass | parent delegates integration continuation | S1 only; later numerical milestones remain Deferred | pending checkpoint handoff |

## Simplicity And Ownership Review

**Applicability:** `applicable`

- Independent concepts and dimensions: recorder persists publications; Euler
  owner advances conservative contents; verifier owns analytical/origin criteria;
  education owns admission; UI selects data. Integration owns shared exports/build.
- State, identity, value, time, policy, and mechanism: accepted state/time/identity
  publish together within each solver. Evaluation/publication times differ.
  Retention never selects numerical acceptance; source admission is no receipt.
- Caller and composition-root knowledge: existing independent APIs stay stable;
  callers do not reconstruct private layouts or reinterpret pressure policies.
  Composition root knows additive exports and explicit archive/API contracts.
- Representative change paths and forced owners: retention changes recorder;
  EOS/method changes solver/oracle; epoch changes education; shared build/export/
  checkpoint changes require integration and affected checks.
- Stable Interfaces versus hidden knowledge: named field/type/unit/association/
  stage/time carry meaning. No latest substitution, undocumented layout,
  checkpoint claim or invented absent field crosses a boundary.
- Independent evolution, testing, failure, and replacement: recorder, solver,
  verifier and UI verify/fail independently. Recording failure discloses tails;
  UI read failure invalidates cache. Composition gets clean-source qualification.
- Necessary complexity and containment: bounded state, numerical admission and
  identity stay in existing owners. No queue, registry, lease, migration,
  fallback solver or extra persistence/replay engine is added.
- Deletion and cumulative machinery result: deleting recording scatters
  durability; deleting Euler scatters EOS/acceptance; deleting independent
  verification loses field admission; deleting this plan loses gate ownership.
  The new epoch is necessary reviewed admission, not a runtime framework.

## Milestones

| Slice/state | Goal and bounded write set | Preserved semantics and gate | Re-plan trigger |
| --- | --- | --- | --- |
| I1 / Accepted | Discover/trial-compose pinned proposals; own plan and lib.rs conflict | Original refs/history/current plan preserved; compatible outcomes reviewed | Changed OID, overlap or missing authority |
| I2 / Accepted | Affected recording/S1/UI tests, bounded composition repair and reports | Contracts, source-pinned S1/primitive capture and independent native software smoke pass | Numerical mismatch, stale value or unsupported display |
| I3 / Accepted | Explicit education epoch/tests/review and fresh complete qualification | Old inventories retained; native/rational/packet and full hosted runtime/book passed | Changed closure, stale receipt or gate failure |
| I4 / Verifying | Own ref/PR/evidence and authorized canonical checkpoint after I3 | Full final-head terminal results/current authority comparison; gate-qualified disposition | Main/component change, unavailable gate or conflicting writer |

## Blockers

Full book and all 18 checks passed at4bcdb7f before standard PR76 merge. Native
functionality passes independently reviewed software display smoke. An early
harness XDestroyWindow exposed a winit panic; normal WM_DELETE_WINDOW closes
successfully. Arbitrary external window/server loss resilience is unclaimed.

Original 8/12/20 acoustic failure remains failed. C1/C2/S1 evidence does not
accept later roadmap claims, S4 recording adapter or vehicle accuracy. Canonical
checkpoint is now recorded after B. Only metadata follow-up review/scoped CI and
disposition remain in this task; main publication pipeline is separately observed.

## Re-Plan Triggers

Changed main/component/plan authority; unsupported/conflicting writes; changed
EOS/method/study/controls/threshold; new dependency/license issue; unexpected
source/publication failure; changed retention/interface promise; unavailable
required observation. Repair within authorization and preserve original evidence.

## Final Acceptance

R/C/B are satisfied for their bounded domains. PR76 merged after all required
gates passed; the canonical S1 release checkpoint is recorded. D remains pending
metadata-only follow-up checks/disposition. Main publication success is not yet claimed.
No component ref or user work is deleted; old failed contexts remain truthful.

- Acceptance status: `partial`
- Final status: `Verifying`
