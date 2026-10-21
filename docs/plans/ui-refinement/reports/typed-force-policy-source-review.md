# Independent typed force policy source review

## Disposition and scope

Suitable for a separate still-gated policy checkpoint. Reviewed preparation began over HEAD `3b3fccea3657d711897130643a478adb48b5edd0` in `/workspace/Rheon-ui-authoring`, against `/tmp/rheon-ui-typed-policy-plan-review.md` (SHA-256 `e696a247a5f540554d4a658203f9a8f55da4d353b95f9b3701c49e250846a759`). Final receipt HEAD is `e5e6cff984d7f220b2b3a741fb8759aa542fccc3`, the separately committed prior runtime evidence; the policy source remains the local increment bound below. `PRODUCTION_QUALIFICATION` is still Pending. No production force enablement, numerical/SQLite rerun or new native acceptance is admitted. No material source blocker remains; focused pure/static receipts are recorded below.

The reviewer read complete changed source/helper paths and wrote only this report. No build, test, solver/model/evaluator/native action, Worker, SQLite operation, output fixture or cleanup was executed. The two earlier authorized Worker results remain bound to their prior source/binary and are not automatically evidence for this changed closure.

## Shared policy and adapters

`force_run_policy` has three explicit Target values and two Qualification values, with one pure check and one production Pending constant. Absent metadata adds no new force restriction; other existing Request validation remains. Present metadata on Unsupported always refuses, even under Validated. Supported targets refuse Pending and allow only the force-eligibility decision under Validated. Domain validity, budgets, immutable member binding and runtime custody remain outside this policy.

Request presence uses any Some in its authoritative per-track snapshot vector, including Some(empty), disabled and zero fields. No field-effect evaluation or fallback to a draft/other member appears. Its target uses the consumed Request workflow: configured Some takes Unsupported precedence, then material Some maps ParticleMaterial, otherwise the existing native carrier branch maps NativeCarrier. The private cfg(test) bypass maps only to Validated, defaults false in ordinary literals and cannot grant Unsupported capability. Non-test compilation uses only the production constant. No public flag, environment selector, serialization field or mutable evidence registry is added.

SceneUi readiness validates/freezes the canonical draft, then invokes the same policy with production Pending; it does not infer intent from the Retained display. Empty ordinary draft remains legacy None semantics, while invalid canonical drafts fail before policy. A retained authored document with an empty draft does not contaminate a new independent setup. Read-only retained projections and viewport/lab refusal remain unchanged.

Visible run controls derive target from effective SetupWorkflow and material mode, so Accepted Session remains Unsupported even if a retained material preference is true. Private Preview start recomputes its consumed Preview/material target and rechecks before freeze/output; start_configured uses static Unsupported before provider/output preparation. The AI entry also uses static Unsupported rather than inheriting Preview capability. Existing configured/native publish/rerun paths still validate the actual immutable Request before their output boundaries; native saved rerun remains independent of current SceneUi/draft. No recording/core/per-track dispatch owner changes in this slice.

## Error ordering note

The initial source applied the authored particle single-track guard even when configured Some made the effective target Unsupported, causing a safe but misleading particle-track refusal. The repaired source derives the effective target and applies that cardinality guard only to ParticleMaterial. A new pure configured+material+two-track assertion requires Unsupported; the helper path was re-inspected and re-admitted. This resolves the message-order finding. Canonical document/member validation appropriately retains precedence over policy errors; a successful policy must never bypass it.

The Pending copy now accurately says native interaction acceptance is pending, preserving the gate after the two tiny Worker results. An existing pure test still expected the former “validation is pending” substring: its first execution failed at that stale assertion, not eligibility behavior. `/tmp/rheon-ui-force-policy-exact4.log` is preserved. The owner changed only that assertion to exact shared `PENDING` error equality, leaving all helpers and subsequent binding checks unchanged; the affected fourth and unexecuted fifth exact paths were re-inspected/re-admitted after list4. No numerical case was rerun.

Initial strict Clippy then identified `SceneUi::draft` as unused after direct-field centralization. Rather than suppressing the lint, readiness now calls `self.draft().freeze()` and reads `self.draft().objects`. The accessor is exactly `&self.forces`, with no state, lock or transformation; this is behavior-equivalent canonical-draft access. The source change was independently re-read, final static gates passed, and only the two accessor-dependent pure tests were repeated successfully. The original lint failure remains preserved as `clippy.log`.

## Complete pure-path admission

These five fully qualified names were inspected and admitted only for individually exact execution after enumeration:

- `force_run_policy_tests::force_policy_qualification_never_grants_unsupported_owner`
- `force_run_policy_tests::force_policy_request_and_setup_resolve_effective_targets`
- `force_run_policy_tests::force_policy_scene_readiness_uses_valid_draft_not_retained_display`
- `force_runtime_ui_tests::force_request_track_resolver_gates_before_work_and_binds_saved_members`
- `scene_ui::tests::scene_force_refusal_projects_disabled_run_without_dispatch`

The new helpers construct literal disabled-zero or empty snapshots, scalar Requests and an intentionally incomplete configured revision sentinel. Supported preflight performs only geometry/buffer arithmetic; configured authored input is refused by policy before its invalid sentinel could reach provider/model preparation. SceneUi/default fixture calls validate/freeze, bind metadata and context projection only. Existing member tests use literal serde/hash/metadata validation and no recorder/readers/files. No test calls positive start/rerun, Worker, numerical constructors, history capture or SQLite.

The UI refusal helper constructs App default with worker/history None, paints only `run_controls` into a Context and captures the possible SetupAction as a boolean. It physically presses/releases the disabled Simulate widget but never calls `dispatch_setup_action`, so even a guard regression cannot launch work in this helper. Named help/status widgets add only UI state. App drop has no worker to cancel/join. The test does not compose full setup/provider/AI/viewport owners.

The decision table checks production Pending explicitly, no-metadata passage under both states, supported pending/validated outcomes and non-bypassable Unsupported. Adapter checks cover Some(empty), disabled-zero fields, configured precedence over material, invalid canonical draft and retained-versus-draft isolation. No numerical or native runtime claim follows from these pure paths.

## Remaining gates

Keep production Pending and the existing private test bypass mechanism unchanged until the separately reviewed enabling increment; neither pure tests nor the two prior tiny Worker cases supply native launch/cancellation/gesture acceptance. Future Unknown owners need explicit policy mapping review rather than inheriting native eligibility. Preserve runtime artifacts and prior native/focus/timeline evidence separately. A copied transfer manifest, if later supplied, is a separate evidence receipt; it is not claimed here.

## Final reviewed source identity and receipt

| File | SHA-256 |
| --- | --- |
| `src/bin/desktop/force_run_policy.rs` | `086d4ee8c18ff30f2165b8622a53d649bd9eb8166bbbbcb82dde3a0eb2427cc2` |
| `src/bin/desktop/force_run_policy_tests.rs` | `a518b47b71cb2aff5d7ce46169918c60d534c00ffccdd0ee5c90abcebbdcdebd` |
| `src/bin/desktop/scene_ui.rs` | `73c16415819c5215a797615f5893222c2ec96eebe2bc8d100b95d346bed7d208` |
| `src/bin/rheon-desktop.rs` | `2c614eb7ea5920ee450ffcb344555304146869b332d8f9fe50f8e7a7703f763c` |
| `src/bin/desktop/force_runtime_ui_tests.rs` | `1d242eaed1b416e3dfb2577d5610c4e0d6d0ca5040f80293681ae184090cebb9` |

All log paths in this receipt begin `/tmp/rheon-ui-force-policy-`. Read-only receipt of final `list5.log`: 204 tests, zero benchmarks. `exact1.log`/`exact2.log` supply the unchanged pure decision-table and Request-target passes. `exact4-copy-fixed.log` supplies the legacy Request/member resolver pass after the copy assertion repair. Final `accessor-exact1.log` and `accessor-exact2.log` supply Scene readiness and disabled-run UI passes after the accessor-only source repair. Each accepted log records one pass, zero failures, 203 filtered out. Earlier `exact3.log`/`exact5.log` also passed before that accessor change; they are superseded for those two final-source assertions. This is five distinct pure cases, with narrowly justified affected repeats, not a broad suite or repeated numerical/SQLite execution.

Final `check2.log` passed in 1.15 s and strict production `clippy2.log` in 4.73 s. The owner confirmed final `fmt2.log` exit zero; fmt2/diffcheck logs are empty. Final hash reads match the table above. The preserved stale-copy assertion failure and unused-accessor lint failure are not hidden or described as passes. The reviewer executed no tests/builds/native actions. The source changes and receipts leave production Pending, maintain Unsupported non-bypass and preserve the separately sealed runtime/native evidence boundaries.
