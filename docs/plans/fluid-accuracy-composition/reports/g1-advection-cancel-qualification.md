# G1 advection cancellation receipt repair qualification

**Locally qualified** at published source `70da36d0f4a58c464a6ad32ed2557e3dfb3d8b52`, tree `114ab60ff25f2d637eac1a06a3b02bb99a55bd15`, in draft [PR137](https://github.com/MrScripty/Rheon/pull/137), dependent on PR136 `e56bc7bd31ea8e8121f09cba15eb8a2bc5e514e4`. Local source checkpoint `9c91acc695f84d2c76f9cb41a58aafad5c776ad6` has the identical tree. Report-only changes after qualification do not change the tested source blobs. Frozen [protocol](g1-advection-cancel-protocol.md), [independent review](g1-advection-cancel-review.md), [manifest](g1-advection-cancel-manifest.json), [next readiness](g1-advection-cancel-next-readiness.md).

## Result and scope

The configured Session already returned `Step(Carrier(Advection(Cancelled)))` without committing; the observer incorrectly fell through to `Failed`. The production repair adds three match-arm lines recognizing that exact cancellation. Actual numerical failures remain `Failed`. No numerical formula/order/domain, publication point, pressure-v1 behavior, codec/version or product UI implementation changed. Explicit historical receipt outcomes are not reclassified on decode.

Actual velocity- and tracer-advection cancellation each run at generation zero and after an accepted interval. Tests check the exact typed error, unchanged dense native state/time/stamps/frame, accepted report/provider/interval metadata and accepted receipt Arc, precise partial trace, no publication, original accepted archive row, real RCT2 close/drop/cold reopen and successful retry on the same owner. Tracer cancellation retains completed pressure/projection evidence; incomplete later stages are absent. Authored historical Failed receipts retain Failed in the cold reader. A real nonzero-forcing native pressure `IterationLimit { iterations: 0 }` control stays Failed and unpublished live and cold.

Native egui tests consume actual velocity/tracer cancellations and the actual pressure failure, both live and through the explicit cold terminal action. Existing pointer, keyboard, focus, evidence, cancellation and close/reopen interaction regressions remain passing. This is headless native consumer evidence, not a new native-window or performance run.

## Exact-source checks

Rust 1.92.0, locked dependencies, binary debug info disabled; environment and exact commands/executable hashes are retained in the manifest/raw bundle. All green checks below ran with the exact compiled source at `70da36d`; report-only artifacts do not alter those source blobs.

| Check | Result |
| --- | --- |
| Default library `workflow::` | 26 passed, 0 failed |
| No-default complete library | 85 passed, 0 failed |
| Eight affected configured Session/provider/recording/runtime/comparison contract targets | 99 passed, 0 failed, 3 existing opt-in ignored |
| No-default borrow doctests | 3 passed, 0 failed |
| Native desktop workflow inspector | 13 passed, 0 failed, 2 existing opt-in ignored |
| Strict no-default and desktop all-target Clippy | Both passed |
| Formatting | Passed |

Total: **226 passing test executions, zero failures, five existing ignored**; counts include deliberate default/no-default overlap and are not distinct-test counts. Four new test functions are included. These checks cover old RCP2/RCP3/RCT1 and existing RCP4/RCT2 behavior, plus the existing additive projection profile compatibility/refusal tests. Broader unrelated scientific campaigns were not rerun or newly qualified.

Retained negative evidence: initial tests-only `9bd2c87` had an Arc type compile error (corrected without production edits); completed tests-only `1c23c88` is identified by its full local commit in the manifest. Its core run reproduced two Failed-versus-Cancelled assertions, one for each actual advection stage, while the actual numerical failure control passed. The native consumer likewise reproduced the Failed-versus-Cancelled assertion. Those red runs stop at the first generation-zero assertion; prior-accepted and full cold matrices are green evidence, not claimed red execution. Red executable copies were not retained; red source checkpoints and logs are retained. All green executable copies are retained where Cargo exposes the executable path.

## Review and integration disposition

Independent architecture, persistence/contracts and native-consumer reviews pass at the exact source. The native reviewer requested retaining the UI helper's typed error and original result assertion; both were fixed before final-source qualification. All findings are disposed.

The unchanged source-admission checker still exits 1: `reviewed native source inventory added or missing path`. This is the existing pending new complete-source admission, not a numerical comparison failure. Working, committed and manifest-selected closures independently agree on 140 paths. Relative to PR136 this repair changes only three existing pins (configured Session, workflow recording tests, native inspector tests), with 137 unchanged and no added/removed selected paths. The full read-only map and exact delta are in the manifest. No admission input was modified. Lead must review/pin/qualify the final closure and complete required hosted checks before integration or merge. PR133/136 remained draft and unmerged at assessment. No merge, force update, rebase or branch cleanup was performed.

Raw evidence: `/workspace/rheon-g1-cancel-evidence`; immutable ZIP and hashes are recorded in the manifest. Historical PR136 evidence and all protected branches/checkpoints remain retained. No physical accuracy, GPU, learned/stateful/external candidate, full C1-B/C/R, native-window or performance acceptance is inferred. The next recommendation is a separately admitted finite stateless classical projection selection slice, as recorded in the readiness disposition.
